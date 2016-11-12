
static mut time: Timer = Timer { sec: 0, msec: 0, };

use volatile::Volatile;

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
      let v_msec = Volatile::new(&time.msec);
      let start: u32 = v_msec.load();
      //while volatile_load(&time.msec as *const u32) - start < ms {/* spin */}
      while *v_msec - start < ms {/* spin */}
    }
  }

  pub fn delay_s(s: u32) {
    unsafe {
      let v_sec = Volatile::new(&time.sec);
      let start: u32 = v_sec.load();
      //while volatile_load(&time.sec as *const u32) - start < s {/* spin */}
      while *v_sec - start < s {/* spin */}
    }
  }
}
