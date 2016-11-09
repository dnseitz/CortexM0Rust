
use super::super::Register;
use core::intrinsics::{volatile_load, volatile_store};

#[derive(Copy, Clone)]
pub struct CVR {
  base_addr: u32,
}

impl Register for CVR {
  fn new(base_addr: u32) -> Self {
    CVR { base_addr: base_addr }
  }

  fn base_addr(&self) -> u32 {
    self.base_addr
  }

  fn mem_offset(&self) -> u32 {
    0x8
  }
}

impl CVR {
  pub fn get_current_value(&self) -> u32 {
    let mask = 0xFFFFFF;

    unsafe {
      let reg = self.addr();

      volatile_load(reg) & mask
    }
  }

  pub fn clear_current_value(&self) {
    // A write to this register clears its value to 0
    unsafe {
      let reg = self.addr();

      volatile_store(reg, 1);
    }
  }
}
