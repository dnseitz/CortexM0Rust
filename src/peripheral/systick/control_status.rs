
use super::super::Register;

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
        *reg |= mask;
      }
      else {
        *reg &= !mask;
      }
    }
  }

  pub fn set_interrupt(&self, enable: bool) {
    let mask = 0b1 << 1;

    unsafe {
      let reg = self.addr();
      if enable {
        *reg |= mask;
      }
      else {
        *reg &= !mask;
      }
    }
  }

  /// Returns true if the counter has reached zero since the last time it was checked.
  pub fn did_underflow(&self) -> bool {
    let mask = 0b1 << 16;

    unsafe {
      let reg = self.addr();
      *reg & mask != 0
    }
  }
}
