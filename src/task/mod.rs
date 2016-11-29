
mod list;

use ::volatile::Volatile;
use ::system_control;
use self::list::TaskQueue;
use ::alloc::boxed::Box;
use ::collections::Vec;

static mut init_task: TaskControl = TaskControl::uninitialized("init");

#[no_mangle]
pub static mut current_task: Option<Box<TaskControl>> = None;

// TODO: Wrap task_list in a mutex lock to provide safe access
static mut task_list: TaskQueue = TaskQueue::new();

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

pub fn new_task(code: fn(), stack_depth: u32, priority: Priority, name: &'static str) {
  let mut task = Box::new(TaskControl::new(stack_depth, name));
  task.initialize(code, priority);

  unsafe { task_list.enqueue(task); }
}

#[repr(C)]
pub struct TaskControl {
  stack: u32, /* stack pointer MUST be first field */
  stack_base: u32,
  stack_depth: u32,
  state: State,
  priority: Priority,
  name: &'static str,
  next: Option<Box<TaskControl>>,
}

impl TaskControl {
  fn new(depth: u32, name: &'static str) -> Self {
    let stack_mem: Vec<u8> = Vec::with_capacity(depth as usize);
    let stack = stack_mem.as_ptr() as u32;
    // Don't free the heap space
    ::core::mem::forget(stack_mem);
    TaskControl {
      stack: stack + depth,
      stack_base: stack,
      stack_depth: depth,
      state: State::Embryo,
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
      priority: Priority::Low,
      name: name,
      next: None,
    }
  }

  fn initialize(&mut self, code: fn(), priority: Priority) {
    const INITIAL_XPSR: u32 = 0x0100_0000;
    unsafe {
      let mut stack_mut = Volatile::new(self.stack as *const _);
      // Offset added to account for way MCU uses stack on entry/exit of interrupts
      stack_mut -= 4;
      stack_mut.store(INITIAL_XPSR); /* xPSR */
      stack_mut -= 4;
      stack_mut.store(code as u32); /* PC */
      stack_mut -= 4;
      stack_mut.store(exit_error as u32); /* LR */
      stack_mut -= 20; /* R12, R3, R2, R1 */
      // *stack_mut = params; /* R0 */
      stack_mut -= 32; /* R11..R4 */
      self.stack = stack_mut.as_ptr() as u32;
    }
    self.state = State::Ready;
    self.priority = priority;
  }

  fn is_stack_overflowed(&self) -> bool {
    self.stack <= self.stack_base
  }
}

/// Select a new task to run and switch its context, this function MUST only be called from the
/// PendSV handler, calling it from elsewhere could lead to undefined behavior. It must be exposed
/// publicly so that the compiler doesn't optimize it away when compiling for release.
#[no_mangle]
pub unsafe fn switch_context() {
  match current_task.take() {
    Some(running) => {
      if running.is_stack_overflowed() {
        ::arm::bkpt();
      }
      loop {
        if let Some(new_task) = task_list.dequeue() {
          task_list.enqueue(running);
          current_task = Some(new_task);
          break;
        }
        else {
          // Go to next priority queue
          // If all queues are empty, reschedule current task
          current_task = Some(running);
          break;
        }
      }
    },
    None => panic!("switch_context - current task doesn't exist!"),
  }
}

pub fn start_first_task() {
  unsafe {
    current_task = task_list.dequeue();
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
          "current_task_const_2: .word current_task\n")
        );
  }
}

fn exit_error() -> ! {
  unsafe {
    ::arm::bkpt();
    loop{}
  }
}

pub fn yield_task() {
  let scb = system_control::scb();
  scb.set_pend_sv();
  scb.clear_pend_sv();
}
