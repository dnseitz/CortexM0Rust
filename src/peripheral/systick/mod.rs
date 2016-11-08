
use super::{Peripheral, Register};

mod control_status;
mod reload_value;
mod current_value;

pub fn systick() -> SysTick {
  SysTick::systick()
}

#[derive(Copy, Clone)]
pub struct SysTick {
  mem_addr: u32,
  csr: control_status::CSR,
  rvr: reload_value::RVR,
  cvr: current_value::CVR,
}

impl Peripheral for SysTick {
  fn mem_addr(&self) -> u32 {
    self.mem_addr
  }
}

impl SysTick {
  fn systick() -> Self {
    const SYSTICK_ADDR: u32 = 0xE000E010;
    SysTick {
      mem_addr: SYSTICK_ADDR,
      csr: control_status::CSR::new(SYSTICK_ADDR),
      rvr: reload_value::RVR::new(SYSTICK_ADDR),
      cvr: current_value::CVR::new(SYSTICK_ADDR),
    }
  }

  pub fn enable_counter(&self) {
    self.csr.set_enable(true);
  }

  pub fn disable_counter(&self) {
    self.csr.set_enable(false);
  }

  pub fn enable_interrupts(&self) {
    self.csr.set_interrupt(true);
  }

  pub fn disable_interrupts(&self) {
    self.csr.set_interrupt(false);
  }

  pub fn get_reload_value(&self) -> u32 {
    self.rvr.get_reload_value()
  }

  pub fn set_reload_value(&self, value: u32) {
    self.rvr.set_reload_value(value);
  }

  pub fn get_current_value(&self) -> u32 {
    self.cvr.get_current_value()
  }

  pub fn clear_current_value(&self) {
    self.cvr.clear_current_value();
  }
}
