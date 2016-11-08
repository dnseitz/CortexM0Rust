
static mut time: Timer = Timer { sec: 0, msec: 0, };

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
  pub fn tick() {
    unsafe {
      time.msec += 1;
      if time.msec % 1000 == 0 {
        time.sec += 1;
      }
    }
  }

  pub fn get_current() -> Timer {
    unsafe { time }
  }

  pub fn delay_ms(ms: u32) {
    unsafe {
      let start: u32 = time.msec;
      while time.msec - start < ms {/* spin */}
    }
  }

  pub fn delay_s(s: u32) {
    unsafe {
      let start: u32 = time.sec;
      while time.sec - start < s {/* spin */}
    }
  }
}
