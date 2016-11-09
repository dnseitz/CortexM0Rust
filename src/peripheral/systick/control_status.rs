
use super::super::Register;
use core::intrinsics::{volatile_load, volatile_store};

pub enum ClockSource {
  Reference,
  Processor,
}

/// The control and status register for the SysTick timer
#[derive(Copy, Clone)]
pub struct CSR {
  base_addr: u32,
}

impl Register for CSR {
  fn new(base_addr: u32) -> Self {
    CSR { base_addr: base_addr }
  }

  fn base_addr(&self) -> u32 {
    self.base_addr
  }

  fn mem_offset(&self) -> u32 {
    0x0
  }
}

impl CSR {
  pub fn set_enable(&self, enable: bool) {
    let mask = 0b1 << 0;

    unsafe {
      let reg = self.addr();
      if enable {
        volatile_store(reg, volatile_load(reg) | mask);
      }
      else {
        volatile_store(reg, volatile_load(reg) & !mask);
      }
    }
  }

  pub fn set_interrupt(&self, enable: bool) {
    let mask = 0b1 << 1;

    unsafe {
      let reg = self.addr();
      if enable {
        volatile_store(reg, volatile_load(reg) | mask);
      }
      else {
        volatile_store(reg, volatile_load(reg) & !mask);
      }
    }
  }

  pub fn set_source(&self, source: ClockSource) {
    let mask = 0b1 << 2;

    unsafe {
      let reg = self.addr();
      match source {
        ClockSource::Reference => volatile_store(reg, volatile_load(reg) & !mask),
        ClockSource::Processor => volatile_store(reg, volatile_load(reg) | mask),
      };
    }
  }

  /// Returns true if the counter has reached zero since the last time it was checked.
  pub fn did_underflow(&self) -> bool {
    let mask = 0b1 << 16;

    unsafe {
      let reg = self.addr();
      volatile_load(reg) & mask != 0
    }
  }
}
