// system_control/icsr.rs
// AltOSRust
//
// Created by Daniel Seitz on 11/30/16

use ::peripheral::Register;

#[derive(Copy, Clone)]
pub struct ICSR {
  base_addr: usize,
}

impl Register for ICSR {
  fn new(base_addr: usize) -> Self {
    ICSR { base_addr: base_addr }
  }

  fn base_addr(&self) -> usize {
    self.base_addr
  }

  fn mem_offset(&self) -> usize {
    0x04
  }
}

impl ICSR {
  pub fn set_pend_sv(&self) {
    const PEND_SV_SET: usize = 0b1 << 28;
    unsafe {
      let mut reg = self.addr();
      *reg |= PEND_SV_SET;
    }
  }

  pub fn clear_pend_sv(&self) {
    const PEND_SV_CLEAR: usize = 0b1 << 27;
    unsafe {
      let mut reg = self.addr();
      *reg |= PEND_SV_CLEAR;
    }
  }
}
