// task/mod.rs
// AltOSRust
//
// Created by Daniel Seitz on 11/30/16

pub mod args;
mod task_control;

use self::task_control::{TaskControl, State};
pub use self::task_control::{TaskHandle, Priority};
use timer::Timer;
use self::args::Args;
use queue::{Queue, SyncQueue, Node};
use alloc::boxed::Box;
use core::ops::Index;
use sync::CriticalSection;

const NUM_PRIORITIES: usize = 4;
pub const FOREVER_CHAN: usize = 0;

#[no_mangle]
#[doc(hidden)]
pub static mut CURRENT_TASK: Option<Box<Node<TaskControl>>> = None;

static PRIORITY_QUEUES: [SyncQueue<TaskControl>; NUM_PRIORITIES] = [SyncQueue::new(),
                                                                    SyncQueue::new(), 
                                                                    SyncQueue::new(), 
                                                                    SyncQueue::new()];
static SLEEP_QUEUE: SyncQueue<TaskControl> = SyncQueue::new();
static DELAY_QUEUE: SyncQueue<TaskControl> = SyncQueue::new();
static OVERFLOW_DELAY_QUEUE: SyncQueue<TaskControl> = SyncQueue::new();

impl Index<Priority> for [SyncQueue<TaskControl>] {
  type Output = SyncQueue<TaskControl>;
  fn index(&self, idx: Priority) -> &Self::Output {
    &self[idx as usize]
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
      if running.destroy {
        drop(running);
      }
      else {
        let queue_index = running.priority;
        if running.is_stack_overflowed() {
          panic!("switch_context - The current task's stack overflowed!");
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
      }

      'main: loop {
        for i in Priority::all() {
          while let Some(mut new_task) = PRIORITY_QUEUES[i].dequeue() {
            if new_task.destroy {
              drop(new_task);
            }
            else {
              new_task.state = State::Running;
              CURRENT_TASK = Some(new_task);
              break 'main;
            }
          }
        }
      }
    },
    None => panic!("switch_context - current task doesn't exist!"),
  }
}

/// Create a new task and put it into the task queue for running. The stack depth is how many bytes
/// should be allocated for the stack, if there is not enough space to allocate the stack the
/// kernel will panic with an out of memory (oom) error.
#[inline(never)]
pub fn new_task(code: fn(&Args), args: Args, stack_depth: usize, priority: Priority, name: &'static str) -> TaskHandle {
  // Make sure the task is allocated in one fell swoop
  let critical_guard = CriticalSection::begin();
  let task = Box::new(Node::new(TaskControl::new(code, args, stack_depth, priority, name)));
  drop(critical_guard);

  let handle = TaskHandle::new(&**task);
  PRIORITY_QUEUES[task.priority].enqueue(task); 
  handle
}

/// Yield the current task to the scheduler so another task can run.
pub fn yield_task() {
  unsafe { ::yield_cpu() };
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
pub fn start_scheduler() {
    init_idle_task();
    unsafe {
      for i in Priority::all() {
        if let Some(mut task) = PRIORITY_QUEUES[i].dequeue() {
          task.state = State::Running;
          CURRENT_TASK = Some(task);
          break;
        }
      }
      debug_assert!(CURRENT_TASK.is_some());
      ::start_first_task();
    }
}

fn is_kernel_running() -> bool {
  unsafe { ::in_kernel_mode() }
}

fn alarm_wake() {
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

fn init_idle_task() {
  let task = TaskControl::new(idle_task_code, Args::empty(), 256, Priority::__Idle, "idle");

  PRIORITY_QUEUES[task.priority].enqueue(Box::new(Node::new(task)));
}

fn idle_task_code(_args: &Args) {
  loop {
    #[cfg(target_arch="arm")]
    unsafe {
      asm!("wfi");
    }
    yield_task();
  }
}
