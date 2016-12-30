// peripheral/systick/current_value.rs
// AltOSRust
//
// Created by Daniel Seitz on 11/30/16

use super::super::Register;

#[derive(Copy, Clone)]
pub struct CVR {
  base_addr: usize,
}

impl Register for CVR {
  fn new(base_addr: usize) -> Self {
    CVR { base_addr: base_addr }
  }

  fn base_addr(&self) -> usize {
    self.base_addr
  }

  fn mem_offset(&self) -> usize {
    0x8
  }
}

impl CVR {
  pub fn get_current_value(&self) -> usize {
    let mask = 0xFFFFFF;

    unsafe {
      let reg = self.addr();

      *reg & mask
    }
  }

  pub fn clear_current_value(&self) {
    // A write to this register clears its value to 0
    unsafe {
      let mut reg = self.addr();

      reg.store(1);
    }
  }
}
