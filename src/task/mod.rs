// task/mod.rs
// AltOSRust
//
// Created by Daniel Seitz on 11/30/16

mod args;

pub use self::args::{ArgsBuilder, Args, Empty};
use volatile::Volatile;
use queue::{AtomicQueue, Node};
use alloc::boxed::Box;
use collections::Vec;
pub use self::imp::*;
use self::priv_imp::*;
use core::ops::Index;
use core::any::Any;

const VALID_TASK: usize = 0xBADB0100;
const INVALID_TASK: usize = 0x0;
const NUM_PRIORITIES: usize = 4;
pub const FOREVER_CHAN: usize = 0;

#[no_mangle]
#[doc(hidden)]
pub static mut CURRENT_TASK: Option<Box<Node<TaskControl>>> = None;

static PRIORITY_QUEUES: [AtomicQueue<TaskControl>; NUM_PRIORITIES] = [AtomicQueue::new(),
                                                                      AtomicQueue::new(), 
                                                                      AtomicQueue::new(), 
                                                                      AtomicQueue::new()];
static SLEEP_QUEUE: AtomicQueue<TaskControl> = AtomicQueue::new();
static DELAY_QUEUE: AtomicQueue<TaskControl> = AtomicQueue::new();
static OVERFLOW_DELAY_QUEUE: AtomicQueue<TaskControl> = AtomicQueue::new();
static mut INIT_TASK: TaskControl = TaskControl::uninitialized("idle");

impl Index<Priority> for [AtomicQueue<TaskControl>] {
  type Output = AtomicQueue<TaskControl>;
  fn index(&self, idx: Priority) -> &Self::Output {
    &self[idx as usize]
  }
}

pub struct TaskHandle(*const TaskControl);

impl TaskHandle {
  fn new(task: *const TaskControl) -> Self {
    TaskHandle(task)
  }
}

#[derive(Copy, Clone)]
pub enum Priority {
  Critical = 0,
  Normal = 1,
  Low = 2,
  Idle = 3,
}

impl Priority {
  fn all() -> ::core::ops::Range<usize> {
    (0..NUM_PRIORITIES)
  }

  fn higher(&self) -> ::core::ops::Range<usize> {
    0..(*self as usize + 1)
  }
}

#[derive(Copy, Clone, PartialEq)]
enum State {
  Embryo,
  Ready,
  Running,
  Blocked,
  Suspended,
}

#[repr(C)]
pub struct TaskControl {
  stack: usize, /* stack pointer MUST be first field */
  stack_base: usize,
  stack_depth: usize,
  tid: usize,
  name: &'static str,
  valid: usize,
  wchan: usize,
  delay: usize,
  overflowed: bool,
  priority: Priority,
  state: State,
}

unsafe impl Send for TaskControl {}

impl TaskControl {
  fn new<T: Any>(code: fn(&Args<T>), args: Args<T>, depth: usize, priority: Priority, name: &'static str) -> Self {
    let stack_mem: Vec<u8> = Vec::with_capacity(depth);
    let stack = stack_mem.as_ptr() as usize;
    // Don't free the heap space
    ::core::mem::forget(stack_mem);
    let tid = tid::fetch_next_tid();
    let mut task = TaskControl {
      stack: stack + depth,
      stack_base: stack,
      stack_depth: depth,
      tid: tid,
      name: name,
      valid: VALID_TASK + (tid & 0xFF),
      wchan: 0,
      delay: 0,
      overflowed: false,
      priority: priority,
      state: State::Embryo,
    };
    task.initialize(code, args);
    task
  }

  const fn uninitialized(name: &'static str) -> Self {
    TaskControl {
      stack: 0,
      stack_base: 0,
      stack_depth: 0,
      tid: !0,
      name: name,
      valid: INVALID_TASK,
      wchan: 0,
      delay: 0,
      overflowed: false,
      priority: Priority::Low,
      state: State::Embryo,
    }
  }

  /// This initializes the task's stack. This method MUST only be called once, calling it more than
  /// once could at best waste some stack space and at worst corrupt an active stack.
  fn initialize<T: Any>(&mut self, code: fn(&Args<T>), args: Args<T>) {
    const INITIAL_XPSR: usize = 0x0100_0000;
    // The Args struct is stored right above the stack
    let args_mem: Box<Args<T>> = Box::new(args);
    let args_ptr = args_mem.as_ptr();
    // Don't deallocate arguments
    ::core::mem::forget(args_mem);
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
      stack_mut.store(args_ptr as usize); /* R0 */
      stack_mut -= 32; /* R11..R4 */
      self.stack = stack_mut.as_ptr() as usize;
    }
    self.state = State::Ready;
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
#[doc(hidden)]
pub unsafe fn switch_context() {
  if !is_kernel_running() {
    panic!("switch_context - This function should only get called from kernel code!");
  }
  match CURRENT_TASK.take() {
    Some(mut running) => {
      let queue_index = running.priority;
      if running.is_stack_overflowed() {
        ::arm::asm::bkpt();
      }
      if running.state == State::Blocked {
        if running.wchan != FOREVER_CHAN {
          SLEEP_QUEUE.enqueue(running);
        }
        else {
          if running.overflowed {
            OVERFLOW_DELAY_QUEUE.enqueue(running);
          }
          else {
            DELAY_QUEUE.enqueue(running);
          }
        }
      }
      else {
        running.state = State::Ready;
        PRIORITY_QUEUES[queue_index].enqueue(running);
      }

      'main: loop {
        for i in Priority::all() {
          if let Some(mut new_task) = PRIORITY_QUEUES[i].dequeue() {
            new_task.state = State::Running;
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
    ::arm::asm::bkpt();
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
  use super::{SLEEP_QUEUE, PRIORITY_QUEUES, CURRENT_TASK};
  use super::{State, Priority, TaskControl, TaskHandle};
  use super::priv_imp::*;
  use system_control;
  use alloc::boxed::Box;
  use queue::{Queue, Node};
  use timer::Timer;
  use super::args::Args;
  use core::any::Any;

  /// Create a new task and put it into the task queue for running. The stack depth is how many bytes
  /// should be allocated for the stack, if there is not enough space to allocate the stack the
  /// kernel will panic with an out of memory (oom) error.
  #[inline(never)]
  pub fn new_task<T: Any>(code: fn(&Args<T>), args: Args<T>, stack_depth: usize, priority: Priority, name: &'static str) -> TaskHandle {
    atomic! {
      {
        let task = Box::new(Node::new(TaskControl::new(code, args, stack_depth, priority, name)));
        let handle = TaskHandle::new(&**task);
        PRIORITY_QUEUES[task.priority].enqueue(task); 
        handle
      }
    }
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


  pub fn wake(wchan: usize) {
    let to_wake: Queue<TaskControl> = SLEEP_QUEUE.remove(|task| task.wchan == wchan);
    for mut task in to_wake.into_iter() {
      task.wchan = 0;
      task.state = State::Ready;
      PRIORITY_QUEUES[task.priority].enqueue(task);
    }
  }
  
  #[doc(hidden)]
  pub fn system_tick() {
    if !is_kernel_running() {
      panic!("alarm_wake - This function should only be called from kernel code!");
    }

    Timer::tick();
    alarm_wake();

    let current_priority = unsafe { 
      match CURRENT_TASK.as_ref() {
        Some(task) => task.priority,
        None => panic!("system_tick - current task doesn't exist!"),
      }
    };
    
    for i in current_priority.higher() {
      if !PRIORITY_QUEUES[i].is_empty() {
        // Only context switch if there's another task at the same or higher priority level
        yield_task();
        break;
      }
    }
  }

  /// Start running the first task in the queue
  pub fn start_first_task() {
    unsafe {
      init_idle_task();
      for i in Priority::all() {
        if let Some(mut task) = PRIORITY_QUEUES[i].dequeue() {
          task.state = State::Running;
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
  use super::{PRIORITY_QUEUES, DELAY_QUEUE, OVERFLOW_DELAY_QUEUE};
  use super::{TaskControl, Priority, State};
  use super::args::{Args, Empty};
  use alloc::boxed::Box;
  use queue::{Queue, Node};
  use timer::Timer;

  const MAIN_STACK: usize = 0b0;
  const PROGRAM_STACK: usize = 0b10;

  #[allow(unused_assignments)] // So testing doesn't have uninitialized variable error
  pub fn is_kernel_running() -> bool {
    unsafe {
      let mut stack_mask: usize = 0;
      #[cfg(target_arch="arm")]
      asm!("mrs $0, CONTROL\n" /* get the stack control mask */
        : "=r"(stack_mask)
        : /* no inputs */
        : /* no clobbers */
        : "volatile");
      stack_mask == MAIN_STACK
    }
  }

  pub fn alarm_wake() {
    if !is_kernel_running() {
      panic!("alarm_wake - This function should only be called from kernel code!");
    }

    let ticks = Timer::get_current().msec;
    
    let to_wake: Queue<TaskControl> = DELAY_QUEUE.remove(|task| task.delay <= ticks);
    for mut task in to_wake.into_iter() {
      task.wchan = 0;
      task.state = State::Ready;
      task.delay = 0;
      PRIORITY_QUEUES[task.priority].enqueue(task);
    }

    if ticks == !0 {
      let mut overflowed: Queue<TaskControl> = OVERFLOW_DELAY_QUEUE.remove_all();
      for task in overflowed.iter_mut() {
        task.overflowed = false;
      }
      DELAY_QUEUE.append(overflowed);
    }
  }

  pub fn init_idle_task() {
    let task = TaskControl::new(init_task_code, Args::empty(), 256, Priority::Idle, "idle");

    PRIORITY_QUEUES[task.priority].enqueue(Box::new(Node::new(task)));
  }

  fn init_task_code(_args: &Args<Empty>) {
    loop {
      #[cfg(target_arch="arm")]
      unsafe {
        asm!("wfi");
      }
      super::yield_task();
    }
  }
}
