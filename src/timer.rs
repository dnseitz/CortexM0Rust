// timer.rs
// AltOSRust
//
// Created by Daniel Seitz on 11/30/16

use volatile::Volatile;
use queue::Queue;
use task;
use task::TaskControl;

static mut TIME: Timer = Timer::new();

#[derive(Copy, Clone)]
pub struct Timer {
  pub sec: usize,
  pub msec: usize,
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

  pub fn delay_ms(ms: usize) {
    unsafe {
      let v_msec = Volatile::new(&TIME.msec);
      let start: usize = v_msec.load();
      let mut remaining = *v_msec - start;
      while remaining < ms {
        task::sleep_for(&TIME as *const _ as usize, ms - remaining);
        remaining = *v_msec - start;
      }
    }
  }

  pub fn delay_s(s: usize) {
    Self::delay_ms(s * 1000);
  }
}
