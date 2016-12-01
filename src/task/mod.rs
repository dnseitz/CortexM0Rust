
mod list;

use volatile::Volatile;
use system_control;
use self::list::TaskQueue;
use alloc::boxed::Box;
use collections::Vec;

#[no_mangle]
pub static mut CURRENT_TASK: Option<Box<TaskControl>> = None;

// TODO: Wrap task_list in a mutex lock to provide safe access
static mut TASK_LIST: TaskQueue = TaskQueue::new();

/*
struct TaskHandle<'task> {
  task: &'task TaskControl,
}
*/

#[derive(Copy, Clone)]
pub enum Priority {
  Critical,
  Low,
}

#[derive(Copy, Clone)]
enum State {
  Ready,
  Running,
  Blocked,
  Suspended,
  Embryo,
}

/// Create a new task and put it into the task queue for running. The stack depth is how many bytes
/// should be allocated for the stack, if there is not enough space to allocate the stack the
/// kernel will panic with an out of memory (oom) error.
pub fn new_task(code: fn(), stack_depth: usize, priority: Priority, name: &'static str) {
  let mut task = Box::new(TaskControl::new(stack_depth, name));
  task.initialize(code, priority);

  unsafe { TASK_LIST.enqueue(task); }
}

#[repr(C)]
pub struct TaskControl {
  stack: usize, /* stack pointer MUST be first field */
  stack_base: usize,
  stack_depth: usize,
  state: State,
  tid: usize,
  priority: Priority,
  name: &'static str,
  next: Option<Box<TaskControl>>,
}

impl TaskControl {
  fn new(depth: usize, name: &'static str) -> Self {
    let stack_mem: Vec<u8> = Vec::with_capacity(depth);
    let stack = stack_mem.as_ptr() as usize;
    // Don't free the heap space
    ::core::mem::forget(stack_mem);
    TaskControl {
      stack: stack + depth,
      stack_base: stack,
      stack_depth: depth,
      state: State::Embryo,
      tid: !0,
      priority: Priority::Critical,
      name: name,
      next: None,
    }
  }

  const fn uninitialized(name: &'static str) -> Self {
    TaskControl {
      stack: 0,
      stack_base: 0,
      stack_depth: 0,
      state: State::Embryo,
      tid: !0,
      priority: Priority::Low,
      name: name,
      next: None,
    }
  }

  /// This initializes the task's stack. This method MUST only be called once, calling it more than
  /// once could at best waste some stack space and at worst corrupt an active stack.
  fn initialize(&mut self, code: fn(), priority: Priority) {
    const INITIAL_XPSR: usize = 0x0100_0000;
    unsafe {
      let mut stack_mut = Volatile::new(self.stack as *const usize);
      // Offset added to account for way MCU uses stack on entry/exit of interrupts
      stack_mut -= 4;
      stack_mut.store(INITIAL_XPSR); /* xPSR */
      stack_mut -= 4;
      stack_mut.store(code as usize); /* PC */
      stack_mut -= 4;
      stack_mut.store(exit_error as usize); /* LR */
      stack_mut -= 20; /* R12, R3, R2, R1 */
      // *stack_mut = params; /* R0 */
      stack_mut -= 32; /* R11..R4 */
      self.stack = stack_mut.as_ptr() as usize;
    }
    self.state = State::Ready;
    self.priority = priority;
    self.tid = tid::fetch_next_tid();
  }

  /// Check if the stack has gone past its bounds
  fn is_stack_overflowed(&self) -> bool {
    // TODO: Add some stack guard bytes to check if we've overflowed during execution?
    //  This would add some extra overhead, maybe have some #[cfg] that determines if we should add
    //  this extra security?
    self.stack <= self.stack_base
  }
}

/// Select a new task to run and switch its context, this function MUST only be called from the
/// PendSV handler, calling it from elsewhere could lead to undefined behavior. It must be exposed
/// publicly so that the compiler doesn't optimize it away when compiling for release.
#[no_mangle]
pub unsafe fn switch_context() {
  match CURRENT_TASK.take() {
    Some(running) => {
      if running.is_stack_overflowed() {
        ::arm::bkpt();
      }
      loop {
        if let Some(new_task) = TASK_LIST.dequeue() {
          TASK_LIST.enqueue(running);
          CURRENT_TASK = Some(new_task);
          break;
        }
        else {
          // Go to next priority queue
          // If all queues are empty, reschedule current task
          CURRENT_TASK = Some(running);
          break;
        }
      }
    },
    None => panic!("switch_context - current task doesn't exist!"),
  }
}

/// Start running the first task in the queue
pub fn start_first_task() {
  unsafe {
    CURRENT_TASK = TASK_LIST.dequeue();
    if CURRENT_TASK.is_none() {
      panic!("start_first_task - tried to start running tasks when no tasks have been created!");
    }
    #[cfg(target_arch="arm")]
    asm!(
      concat!(
          "ldr r2, current_task_const_2\n", /* get location of current_task */
          "ldr r3, [r2]\n",
          "ldr r0, [r3]\n",

          "adds r0, #32\n", /* discard everything up to r0 */
          "msr psp, r0\n", /* this is the new top of stack to use for the task */

          "movs r0, #2\n", /* switch to the psp stack */
          "msr CONTROL, r0\n", /* we're using psp instead of msp now */

          "isb\n", /* instruction barrier */

          "pop {r0-r5}\n", /* pop the registers that are saved automatically */
          "mov lr, r5\n", /* lr is now in r5, so put it back where it belongs */
          "pop {r3}\n", /* pop return address (old pc) into r3 */
          "pop {r2}\n", /* pop and discard xPSR */
          "cpsie i\n", /* first task has its context, so interrupts can be enabled */
          "bx r3\n", /* start executing user code */

           ".align 4\n",
          "current_task_const_2: .word CURRENT_TASK\n")
      : /* no outputs */
      : /* no inputs */
      : /* no clobbers */
      : "volatile");
  }
}

fn exit_error() -> ! {
  unsafe {
    ::arm::bkpt();
    loop{}
  }
}

/// Yield the current task to the scheduler so another task can run.
pub fn yield_task() {
  let scb = system_control::scb();
  scb.set_pend_sv();
  scb.clear_pend_sv();
}

mod tid {
  use atomic::Atomic;

  static CURRENT_TID: Atomic<usize> = Atomic::new(0);
  
  /// Atomically increment the task id and return the old value
  pub fn fetch_next_tid() -> usize {
    CURRENT_TID.fetch_add(1)
  }
}
