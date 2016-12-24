// timer.rs
// AltOSRust
//
// Created by Daniel Seitz on 11/30/16

//! System time handling.
//!
//! This module helps keep track of the system time and how much time has passed.

use syscall;
use atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};
use sync::CriticalSection;
use core::ops::{Add, AddAssign, Sub};
use core::cell::UnsafeCell;

static SYSTEM_TIME: Tick = Tick::new();

const DEFAULT_RESOLUTION: usize = 1;

/// Tick the system tick counter
///
/// This method should only be called by the system tick interrupt handler.
pub fn tick() {
  SYSTEM_TIME.tick();
}

/// Return the number of ticks that have passed since the system started.
///
/// The ticks can overflow and wrap back to 0, so the value returned is not guaranteed to be
/// greater than the previous values.
pub fn get_tick() -> usize {
  SYSTEM_TIME.get_tick()
}

/// Set the ms resolution of the ticks.
///
/// This should only be called once upon initialization of the system. Setting this after the
/// system has been running for a while could cause some tasks that are delayed to wake up too
/// early.
///
/// # Example
///
/// ```rust,no_run
/// use altos_core::time::{self, Time};
///
/// // Assuming we tick 2 times every ms...
/// time::set_resolution(2);
/// // now every other tick we will increment the system timer by 1 ms
///
/// time::tick();
/// time::tick();
///
/// assert_eq!(Time::now().msec, 1);
/// ```
pub fn set_resolution(ticks_per_ms: usize) {
  SYSTEM_TIME.set_resolution(ticks_per_ms);
}

/// Type to keep track of how many ticks have passed since the start of the system.
struct Tick {
  ticks: AtomicUsize,
  ms_res: AtomicUsize,
  time: UnsafeCell<Time>,
}

unsafe impl Sync for Tick {}

impl Tick {
  const fn new() -> Self {
    Tick {
      ticks: ATOMIC_USIZE_INIT,
      ms_res: AtomicUsize::new(DEFAULT_RESOLUTION),
      time: UnsafeCell::new(Time::new()),
    }
  }

  /// Tick the system tick counter
  ///
  /// This method should only be called by the system tick interrupt handler.
  #[doc(hidden)]
  fn tick(&self) {
    // TODO: Come back and examine more closely to see if we need this to be an atomic operation.
    //  Since we're in the system tick handler we know we wont get preemted by anything other than
    //  higher priority interrupts.
    let old_ticks = self.ticks.fetch_add(1, Ordering::Relaxed);
    let res = self.ms_res.load(Ordering::Relaxed);
    if res != 0 && (old_ticks + 1) % res == 0 {
      unsafe { (&mut *self.time.get()).increment() };
    }
  }

  /// Return the number of ticks that have passed since the system started.
  ///
  /// The ticks can overflow and wrap back to 0, so the value returned is not guaranteed to be
  /// greater than the previous values.
  fn get_tick(&self) -> usize {
    self.ticks.load(Ordering::Relaxed)
  }

  /// Set the ms resolution of the ticks.
  ///
  /// This should only be called once upon initialization of the system. Setting this after the
  /// system has been running for a while could cause some tasks that are delayed to wake up too
  /// early.
  // TODO: Potentially we could update any delayed tasks with the new resolution, but this seems
  //  like it would only be useful in very *very* specific cases and could cause a lot of added
  //  complexity to the code.
  fn set_resolution(&self, ticks_per_ms: usize) {
    debug_assert!(ticks_per_ms > 0);
    self.ms_res.store(ticks_per_ms, Ordering::SeqCst);
  }

  fn get_resolution(&self) -> usize {
    self.ms_res.load(Ordering::Relaxed)
  }

  fn time(&self) -> Time {
    unsafe { *self.time.get() }
  }
}

/// A type containing information about the time passed since the start of the system.
#[derive(Copy, Clone)]
pub struct Time {
  /// Number of seconds that have passed.
  pub sec: usize,
  
  /// Number of milliseconds that have passed.
  pub msec: usize,
}

impl Time {
  /// Create a new timer initialized to 0 sec, 0 msec.
  const fn new() -> Self {
    Time {
      sec: 0,
      msec: 0,
    }
  }

  /// Get the current system time.
  pub fn now() -> Time {
    let _g = CriticalSection::begin();
    SYSTEM_TIME.time()
  }

  /// Delay a task for a certain number of milliseconds.
  ///
  /// This method takes a `usize` argument for the number of milliseconds to delay the currently
  /// running task.
  pub fn delay_ms(ms: usize) {
    syscall::sleep_for(syscall::FOREVER_CHAN, ms * SYSTEM_TIME.get_resolution());
  }

  /// Delay a task for a certain number of seconds.
  ///
  /// This method takes a `usize` argument for the number of seconds to delay the currently running
  /// task.
  pub fn delay_s(s: usize) {
    Self::delay_ms(s * 1000);
  }

  /// Increment the system time by 1 ms, incrementing the seconds as well if our ms rolls over.
  fn increment(&mut self) {
    let increment = Time {
      sec: 0,
      msec: 1,
    };
    let _g = CriticalSection::begin();
    *self += increment;
  }
}

impl Add<Time> for Time {
  type Output = Time;

  fn add(mut self, rhs: Time) -> Self::Output {
    self.sec += rhs.sec;
    self.msec += rhs.msec;
    if self.msec >= 1000 {
      self.sec += 1;
      self.msec %= 1000;
    }
    self
  }
}

impl AddAssign<Time> for Time {
  fn add_assign(&mut self, rhs: Time) {
    *self = *self + rhs;
  }
}

impl Sub<Time> for Time {
  type Output = Time;

  fn sub(mut self, rhs: Time) -> Self::Output {
    // TODO: Figure out how to handle subtracting a bigger time from a smaller time... represent
    //  seconds as an isize instead? (Then we lose a lot of our number space, and we have to check
    //  for overflow)
    self.sec -= rhs.sec;
    if self.msec > rhs.msec {
      self.msec -= rhs.msec;
    }
    else {
      self.sec -= 1;
      self.msec = 1000 - (rhs.msec - self.msec)
    }
    self
  }
}

#[cfg(test)]
mod tests {
  use super::{Tick, Time};

  #[test]
  fn smoke() {
    let tick = Tick::new();

    tick.tick();

    assert_eq!(tick.get_tick(), 1);
  }

  #[test]
  fn smoke2() {
    let tick = Tick::new();

    tick.tick();
    tick.tick();

    assert_eq!(tick.get_tick(), 2);
  }

  #[test]
  fn two_msec_resolution_ticks() {
    let tick = Tick::new();

    tick.set_resolution(2);

    tick.tick();

    assert_eq!(tick.time().msec, 0);

    tick.tick();

    assert_eq!(tick.time().msec, 1);

  }

  #[test]
  fn sec_ticks() {
    let tick = Tick::new();

    tick.set_resolution(1);

    for _ in 0..1000 {
      tick.tick();
    }

    assert_eq!(tick.time().sec, 1);
    assert_eq!(tick.time().msec, 0);
  }

  #[test]
  fn sec_ticks_2() {
    let tick = Tick::new();

    tick.set_resolution(1);

    for _ in 0..1500 {
      tick.tick();
    }

    assert_eq!(tick.time().sec, 1);
    assert_eq!(tick.time().msec, 500);
  }

  #[test]
  fn add_times() {
    let time1 = Time { sec: 100, msec: 10 };
    let time2 = Time { sec: 200, msec: 20 };

    let time3 = time1 + time2;

    assert_eq!(time3.sec, 300);
    assert_eq!(time3.msec, 30);
  }

  #[test]
  fn add_times_overflowing() {
    let time1 = Time { sec: 100, msec: 900 };
    let time2 = Time { sec: 100, msec: 200 };

    let time3 = time1 + time2;

    assert_eq!(time3.sec, 201);
    assert_eq!(time3.msec, 100);
  }
}
