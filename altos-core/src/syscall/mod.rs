// syscall/mod.rs
// AltOSRust
//
// Created by Daniel Seitz on 12/20/16

//! Syscall interface for the AltOS kernel

use task::{CURRENT_TASK, DELAY_QUEUE, OVERFLOW_DELAY_QUEUE, PRIORITY_QUEUES};
use task::State;
use timer::Timer;
use sync::CriticalSection;

/// An alias for the channel to sleep on that will never be awoken
pub const FOREVER_CHAN: usize = 0;

/// Yield the current task to the scheduler so another task can run.
///
/// # Examples
///
/// ```rust,no_run
/// use altos_core::syscall::sched_yield;
/// use altos_core::Args;
///
/// fn test_task(_args: &Args) {
///   loop {
///     // Do some important work...
///   
///     // Okay, we're done...
///     sched_yield();
///     // Go back and do it again
///   }
/// }
/// ```
pub fn sched_yield() {
  unsafe { ::yield_cpu() };
}

/// Put the current task to sleep, waiting on a channel to be woken up.
///
/// `sleep` takes a `usize` argument that acts as an identifier for when to wake up the task. The
/// task will sleep indefinitely if no wakeup signal is sent.
///
/// # Examples
///
/// ```no_run
/// use altos_core::syscall::sleep;
/// use altos_core::atomic::{AtomicBool, ATOMIC_BOOL_INIT, Ordering};
///
/// static flag: AtomicBool = ATOMIC_BOOL_INIT;
///
/// while !flag.load(Ordering::SeqCst) {
///   // Block until some other thread wakes us up
///   sleep(&flag as *const _ as usize);
/// }
/// ```
pub fn sleep(wchan: usize) {
  sleep_for(wchan, 0);
}

/// Put the current task to sleep with a timeout, waiting on a channel to be woken up.
///
/// `sleep_for` takes a `usize` argument that acts as an identifier to wake up the task. It also
/// takes a second `usize` argument for the maximum time it should sleep before waking.
///
/// # Examples
///
/// ```no_run
/// use altos_core::syscall::{sleep_for, FOREVER_CHAN};
///
/// // Sleep for 300 ticks
/// sleep_for(FOREVER_CHAN, 300);
/// ```
pub fn sleep_for(wchan: usize, delay: usize) {
  let _g = CriticalSection::begin();
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
  sched_yield();
}

/// Wake up all tasks sleeping on a channel.
///
/// `wake` takes a `usize` argument that acts as an identifier to only wake up tasks sleeping on
/// that same identifier. 
pub fn wake(wchan: usize) {
  let _g = CriticalSection::begin();
  let mut to_wake = DELAY_QUEUE.remove(|task| task.wchan == wchan);
  to_wake.append(OVERFLOW_DELAY_QUEUE.remove(|task| task.wchan == wchan));
  for mut task in to_wake.into_iter() {
    task.wchan = 0;
    task.state = State::Ready;
    PRIORITY_QUEUES[task.priority].enqueue(task);
  }
}

#[doc(hidden)]
pub fn system_tick() {
  debug_assert!(unsafe { ::in_kernel_mode() });

  let _g = CriticalSection::begin();
  Timer::tick();

  // wake up all tasks sleeping until the current tick
  let ticks = Timer::get_current().msec;
  
  let to_wake = DELAY_QUEUE.remove(|task| task.delay <= ticks && task.wchan == FOREVER_CHAN);
  for mut task in to_wake.into_iter() {
    task.wchan = 0;
    task.state = State::Ready;
    task.delay = 0;
    PRIORITY_QUEUES[task.priority].enqueue(task);
  }

  if ticks == !0 {
    let mut overflowed = OVERFLOW_DELAY_QUEUE.remove_all();
    for task in overflowed.iter_mut() {
      task.overflowed = false;
    }
    DELAY_QUEUE.append(overflowed);
  }

  let current_priority = unsafe { 
    match CURRENT_TASK.as_ref() {
      Some(task) => task.priority,
      None => panic!("system_tick - current task doesn't exist!"),
    }
  };
  
  for i in current_priority.higher() {
    if !PRIORITY_QUEUES[i].is_empty() {
      // Only context switch if there's another task at the same or higher priority level
      sched_yield();
      break;
    }
  }
}
