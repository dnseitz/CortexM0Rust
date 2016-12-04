// task/mod.rs
// AltOSRust
//
// Created by Daniel Seitz on 11/30/16

use volatile::Volatile;
use queue::{AtomicQueue, Queueable};
use alloc::boxed::Box;
use collections::Vec;
pub use self::imp::*;
use self::priv_imp::*;

const VALID_TASK: usize = 0xBADB0100;
const INVALID_TASK: usize = 0x0;
const NUM_PRIORITIES: usize = 3;

#[no_mangle]
#[doc(hidden)]
pub static mut CURRENT_TASK: Option<Box<TaskControl>> = None;

static PRIORITY_QUEUES: [AtomicQueue<TaskControl>; NUM_PRIORITIES] = [AtomicQueue::new(), AtomicQueue::new(), AtomicQueue::new()];
static SLEEP_QUEUE: AtomicQueue<TaskControl> = AtomicQueue::new();
static DELAY_QUEUE: AtomicQueue<TaskControl> = AtomicQueue::new();
static OVERFLOW_DELAY_QUEUE: AtomicQueue<TaskControl> = AtomicQueue::new();
static mut INIT_TASK: TaskControl = TaskControl::uninitialized("init");

pub struct TaskHandle(*const TaskControl);

impl TaskHandle {
  fn new(task: *const TaskControl) -> Self {
    TaskHandle(task)
  }
}

#[derive(Copy, Clone)]
pub enum Priority {
  Critical,
  Low,
  Init,
}

impl Priority {
  fn all() -> ::core::ops::Range<usize> {
    (0..NUM_PRIORITIES)
  }

  fn index(&self) -> usize {
    match *self {
      Priority::Critical => 0,
      Priority::Low => 1,
      Priority::Init => 2,
    }
  }
}

#[derive(Copy, Clone, PartialEq)]
enum State {
  Ready,
  Running,
  Blocked,
  Suspended,
  Embryo,
}

#[repr(C)]
#[derive(Clone)]
pub struct TaskControl {
  stack: usize, /* stack pointer MUST be first field */
  stack_base: usize,
  stack_depth: usize,
  state: State,
  tid: usize,
  priority: Priority,
  name: &'static str,
  valid: usize,
  wchan: usize,
  delay: usize,
  overflowed: bool,
  next: Option<Box<TaskControl>>,
}

unsafe impl Send for TaskControl {}

impl TaskControl {
  fn new(depth: usize, name: &'static str) -> Self {
    let stack_mem: Vec<u8> = Vec::with_capacity(depth);
    let stack = stack_mem.as_ptr() as usize;
    // Don't free the heap space
    ::core::mem::forget(stack_mem);
    let tid = tid::fetch_next_tid();
    TaskControl {
      stack: stack + depth,
      stack_base: stack,
      stack_depth: depth,
      state: State::Embryo,
      tid: tid,
      priority: Priority::Critical,
      name: name,
      valid: VALID_TASK + (tid & 0xFF),
      wchan: 0,
      delay: 0,
      overflowed: false,
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
      valid: INVALID_TASK,
      wchan: 0,
      delay: 0,
      overflowed: false,
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
  }

  /// Check if the stack has gone past its bounds
  fn is_stack_overflowed(&self) -> bool {
    // TODO: Add some stack guard bytes to check if we've overflowed during execution?
    //  This would add some extra overhead, maybe have some #[cfg] that determines if we should add
    //  this extra security?
    self.stack <= self.stack_base
  }
}

impl Queueable for TaskControl {
  fn take_next(&mut self) -> Option<Box<Self>> {
    self.next.take()
  }
  fn set_next(&mut self, next: Option<Box<Self>>) {
    self.next = next;
  }
}

/// Select a new task to run and switch its context, this function MUST only be called from the
/// PendSV handler, calling it from elsewhere could lead to undefined behavior. It must be exposed
/// publicly so that the compiler doesn't optimize it away when compiling for release.
#[no_mangle]
#[doc(hidden)]
pub unsafe fn switch_context() {
  if !is_kernel_running() {
    panic!("switch_context - This function should only get called from kernel code!");
  }
  match CURRENT_TASK.take() {
    Some(running) => {
      let queue_index = running.priority.index();
      if running.is_stack_overflowed() {
        ::arm::bkpt();
      }
      if running.state == State::Blocked {
        if running.delay != 0 {
          if running.overflowed {
            OVERFLOW_DELAY_QUEUE.enqueue(running);
          }
          else {
            DELAY_QUEUE.enqueue(running);
          }
        }
        else {
          SLEEP_QUEUE.enqueue(running);
        }
      }
      else {
        PRIORITY_QUEUES[queue_index].enqueue(running);
      }

      'main: loop {
        for i in Priority::all() {
          if let Some(new_task) = PRIORITY_QUEUES[i].dequeue() {
            CURRENT_TASK = Some(new_task);
            break 'main;
          }
        }
      }
    },
    None => panic!("switch_context - current task doesn't exist!"),
  }
}


fn exit_error() -> ! {
  unsafe {
    ::arm::bkpt();
    loop{}
  }
}

mod tid {
  use atomic::Atomic;

  static CURRENT_TID: Atomic<usize> = Atomic::new(0);
  
  /// Atomically increment the task id and return the old value
  pub fn fetch_next_tid() -> usize {
    CURRENT_TID.fetch_add(1)
  }
}

mod imp {
  use super::{SLEEP_QUEUE, DELAY_QUEUE, OVERFLOW_DELAY_QUEUE, PRIORITY_QUEUES, CURRENT_TASK};
  use super::{State, Priority, TaskControl, TaskHandle};
  use super::priv_imp::*;
  use system_control;
  use alloc::boxed::Box;
  use queue::Queue;
  use timer::Timer;

  /// Create a new task and put it into the task queue for running. The stack depth is how many bytes
  /// should be allocated for the stack, if there is not enough space to allocate the stack the
  /// kernel will panic with an out of memory (oom) error.
  pub fn new_task(code: fn(), stack_depth: usize, priority: Priority, name: &'static str) -> TaskHandle {
    let mut task = Box::new(TaskControl::new(stack_depth, name));
    task.initialize(code, priority);
    let handle = TaskHandle::new(&*task);

    PRIORITY_QUEUES[task.priority.index()].enqueue(task); 
    handle
  }

  /// Yield the current task to the scheduler so another task can run.
  pub fn yield_task() {
    let scb = system_control::scb();
    scb.set_pend_sv();
  }

  pub fn sleep(wchan: usize) {
    sleep_for(wchan, 0);
  }

  pub fn sleep_for(wchan: usize, delay: usize) {
    unsafe {
      if let Some(current) = CURRENT_TASK.as_mut() {
        let ticks = Timer::get_current().msec;
        current.wchan = wchan;
        current.state = State::Blocked;
        current.delay = ticks + delay;
        if ticks + delay < ticks {
          current.overflowed = true;
        }
      }
      else {
        panic!("sleep_for - current task doesn't exist!");
      }
    }
    yield_task();
  }

  pub fn alarm_wake() {
    if !is_kernel_running() {
      panic!("alarm_wake - This function should only be called from kernel code!");
    }

    let ticks = Timer::get_current().msec;
    
    let mut to_wake: Queue<TaskControl> = DELAY_QUEUE.remove(|task| task.delay <= ticks);
    to_wake.modify_all(|task| { task.wchan = 0; task.state = State::Ready; task.delay = 0; });
    while let Some(task) = to_wake.dequeue() {
      PRIORITY_QUEUES[task.priority.index()].enqueue(task);
    }

    if ticks == !0 {
      let mut overflowed: Queue<TaskControl> = OVERFLOW_DELAY_QUEUE.remove_all();
      overflowed.modify_all(|task| task.overflowed = false );
      DELAY_QUEUE.append(overflowed);
    }
  }

  pub fn wake(wchan: usize) {
    let mut to_wake: Queue<TaskControl> = SLEEP_QUEUE.remove(|task| task.wchan == wchan);
    to_wake.modify_all(|task| { task.wchan = 0; task.state = State::Ready; });
    while let Some(task) = to_wake.dequeue() {
      PRIORITY_QUEUES[task.priority.index()].enqueue(task);
    }
  }

  /// Start running the first task in the queue
  pub fn start_first_task() {
    unsafe {
      init_first_task();
      for i in Priority::all() {
        if let Some(task) = PRIORITY_QUEUES[i].dequeue() {
          CURRENT_TASK = Some(task);
          break;
        }
      }
      debug_assert!(CURRENT_TASK.is_some());

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
}

mod priv_imp {
  use super::PRIORITY_QUEUES;
  use super::{INIT_TASK, TaskControl, Priority};
  use alloc::boxed::Box;

  pub fn is_kernel_running() -> bool {
    unsafe {
      const PSP: usize = 1 << 1;
      let mut stack_mask: usize = 0;
      #[cfg(target_arch="arm")]
      asm!("mrs $0, CONTROL\n" /* get the stack control mask */
        : "=r"(stack_mask)
        : /* no inputs */
        : /* no clobbers */
        : "volatile");
      stack_mask & PSP == 0
    }
  }

  pub fn init_first_task() {
    let mut task = TaskControl::new(256, "init");
    task.initialize(init_task_code, Priority::Init);

    unsafe { 
      INIT_TASK = task;
      PRIORITY_QUEUES[INIT_TASK.priority.index()].enqueue(Box::from_raw(&mut INIT_TASK));
    }
  }

  fn init_task_code() {
    loop {
      #[cfg(target_arch="arm")]
      unsafe {
        asm!("wfi");
      }
      super::yield_task();
    }
  }
}
