
static mut time: Timer = Timer { sec: 0, msec: 0, };

use core::intrinsics::{volatile_load, volatile_store};

#[derive(Copy, Clone)]
pub struct Timer {
  sec: u32,
  msec: u32,
}

impl Timer {
  fn new() -> Self {
    Timer {
      sec: 0,
      msec: 0,
    }
  }

  /// Tick by 1 ms
  #[inline(never)]
  pub fn tick() {
    unsafe {
      time.msec += 1;
      if time.msec % 1000 == 0 {
        time.sec += 1;
      }
    }
  }

  #[inline(never)]
  pub fn get_current() -> Timer {
    unsafe { time }
  }

  #[inline(never)]
  pub fn delay_ms(ms: u32) {
    unsafe {
      let start: u32 = time.msec;
      while volatile_load(&time.msec as *const u32) - start < ms {/* spin */}
    }
  }

  #[inline(never)]
  pub fn delay_s(s: u32) {
    unsafe {
      let start: u32 = time.sec;
      while volatile_load(&time.sec as *const u32) - start < s {/* spin */}
    }
  }
}
