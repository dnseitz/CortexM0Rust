// task/mod.rs
// AltOSRust
//
// Created by Daniel Seitz on 11/30/16

//! Task creation, scheduling  and system calls.
//!
//! This module contains the functions used to create tasks and modify them within the kernel. It
//! also contains the code for the scheduler.

pub mod public;
mod args;
mod task_control;

use syscall::sched_yield;
pub use self::task_control::{TaskHandle, Priority, TaskControl, State};
pub use self::args::{ArgsBuilder, Args};
use queue::{SyncQueue, Node};
use alloc::boxed::Box;
use core::ops::Index;
use sync::CriticalSection;

const NUM_PRIORITIES: usize = 4;

/// The current task.
///
/// This keeps track of the currently running task, this should always be `Some` unless the task is
/// actively being switched out or the scheduler has not been started.
#[no_mangle]
#[allow(private_no_mangle_statics)]
#[doc(hidden)]
pub static mut CURRENT_TASK: Option<Box<Node<TaskControl>>> = None;

pub static PRIORITY_QUEUES: [SyncQueue<TaskControl>; NUM_PRIORITIES] = [SyncQueue::new(),
                                                                    SyncQueue::new(), 
                                                                    SyncQueue::new(), 
                                                                    SyncQueue::new()];
pub static DELAY_QUEUE: SyncQueue<TaskControl> = SyncQueue::new();
pub static OVERFLOW_DELAY_QUEUE: SyncQueue<TaskControl> = SyncQueue::new();

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
  /*
  if !is_kernel_running() {
    panic!("switch_context - This function should only get called from kernel code!");
  }
  */
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
          if running.overflowed {
            OVERFLOW_DELAY_QUEUE.enqueue(running);
          }
          else {
            DELAY_QUEUE.enqueue(running);
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

/// Creates a new task and put it into the task queue for running. It returns a `TaskHandle` to
/// monitor the task with
///
/// `new_task` takes several arguments, a `fn(&Args)` pointer which specifies the code to run for
/// the task, an `Args` argument for the arguments that will be passed to the task, a `usize`
/// argument for how much space should be allocated for the task's stack, a `Priority` argument for
/// the priority that the task should run at, and a `&str` argument to give the task a readable
/// name.
///
/// # Examples
///
/// ```rust,no_run
/// use altos_core::{start_scheduler, new_task, Priority};
/// use altos_core::Args;
///
/// // Create the task and hold onto the handle
/// let handle = new_task(test_task, Args::empty(), 512, Priority::Normal, "new_task_name");
///
/// // Start running the task
/// start_scheduler(); 
///
/// fn test_task(_args: &mut Args) {
///   // Do stuff here...
///   loop {}
/// }
/// ```
#[inline(never)]
pub fn new_task(code: fn(&mut Args), args: Args, stack_depth: usize, priority: Priority, name: &'static str) -> TaskHandle {
  // Make sure the task is allocated in one fell swoop
  let g = CriticalSection::begin();
  let task = Box::new(Node::new(TaskControl::new(code, args, stack_depth, priority, name)));
  drop(g);

  let handle = TaskHandle::new(&**task);
  PRIORITY_QUEUES[task.priority].enqueue(task); 
  handle
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

fn init_idle_task() {
  let task = TaskControl::new(idle_task_code, Args::empty(), 256, Priority::__Idle, "idle");

  PRIORITY_QUEUES[task.priority].enqueue(Box::new(Node::new(task)));
}

fn idle_task_code(_args: &mut Args) {
  loop {
    #[cfg(target_arch="arm")]
    unsafe {
      asm!("wfi");
    }
    sched_yield();
  }
}
