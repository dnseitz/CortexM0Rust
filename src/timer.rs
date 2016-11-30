
static mut TIME: Timer = Timer::new();

use volatile::Volatile;

#[derive(Copy, Clone)]
pub struct Timer {
  sec: u32,
  msec: u32,
}

impl Timer {
  const fn new() -> Self {
    Timer {
      sec: 0,
      msec: 0,
    }
  }

  /// Tick by 1 ms
  pub fn tick() {
    unsafe {
      TIME.msec += 1;
      if TIME.msec % 1000 == 0 {
        TIME.sec += 1;
      }
    }
  }

  pub fn get_current() -> Timer {
    unsafe { TIME }
  }

  pub fn delay_ms(ms: u32) {
    unsafe {
      let v_msec = Volatile::new(&TIME.msec);
      let start: u32 = v_msec.load();
      while *v_msec - start < ms {/* spin */}
    }
  }

  pub fn delay_s(s: u32) {
    unsafe {
      let v_sec = Volatile::new(&TIME.sec);
      let start: u32 = v_sec.load();
      while *v_sec - start < s {/* spin */}
    }
  }
}
