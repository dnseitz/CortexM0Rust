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
use core::ops::{Deref, Add, AddAssign, Sub};

static TICKS: Tick = Tick::new();
static mut TIME: Time = Time::new();

const DEFAULT_RESOLUTION: usize = 1;

/// Type to keep track of how many ticks have passed since the start of the system.
pub struct Tick {
  ticks: AtomicUsize,
  ms_res: AtomicUsize,
}

impl Tick {
  const fn new() -> Self {
    Tick {
      ticks: ATOMIC_USIZE_INIT,
      ms_res: AtomicUsize::new(DEFAULT_RESOLUTION),
    }
  }

  /// Tick the system tick counter
  ///
  /// This method should only be called by the system tick interrupt handler.
  #[doc(hidden)]
  pub fn tick() {
    // TODO: Come back and examine more closely to see if we need this to be an atomic operation.
    //  Since we're in the system tick handler we know we wont get preemted by anything other than
    //  higher priority interrupts.
    let old_ticks = TICKS.fetch_add(1, Ordering::Relaxed);
    let res = TICKS.ms_res.load(Ordering::Relaxed);
    if res != 0 && old_ticks % res == 0 {
      Time::increment();
    }
  }

  /// Return the number of ticks that have passed since the system started.
  ///
  /// The ticks can overflow and wrap back to 0, so the value returned is not guaranteed to be
  /// greater than the previous values.
  pub fn get_tick() -> usize {
    TICKS.load(Ordering::Relaxed)
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
  /// use altos_core::timer::Tick;
  ///
  /// // Assuming we tick 2 times every ms...
  /// Tick::set_resolution(2);
  /// // now every other tick we will increment the system timer by 1 ms
  ///
  /// Tick::tick();
  /// Tick::tick();
  ///
  /// assert_eq!(Time::now().ms, 1);
  /// ```
  // TODO: Potentially we could update any delayed tasks with the new resolution, but this seems
  //  like it would only be useful in very *very* specific cases and could cause a lot of added
  //  complexity to the code.
  pub fn set_resolution(ticks_per_ms: usize) {
    debug_assert!(ticks_per_ms > 0);
    TICKS.ms_res.store(ticks_per_ms, Ordering::SeqCst);
  }

  fn get_resolution() -> usize {
    TICKS.ms_res.load(Ordering::Relaxed)
  }
}

impl Deref for Tick {
  type Target = AtomicUsize;

  fn deref(&self) -> &Self::Target {
    &self.ticks
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
    unsafe { TIME }
  }

  /// Delay a task for a certain number of milliseconds.
  ///
  /// This method takes a `usize` argument for the number of milliseconds to delay the currently
  /// running task.
  pub fn delay_ms(ms: usize) {
    syscall::sleep_for(syscall::FOREVER_CHAN, ms * Tick::get_resolution());
  }

  /// Delay a task for a certain number of seconds.
  ///
  /// This method takes a `usize` argument for the number of seconds to delay the currently running
  /// task.
  pub fn delay_s(s: usize) {
    Self::delay_ms(s * 1000);
  }

  /// Increment the system time by 1 ms, incrementing the seconds as well if our ms rolls over.
  fn increment() {
    let increment = Time {
      sec: 0,
      msec: 1,
    };
    let _g = CriticalSection::begin();
    unsafe { TIME += increment };
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
