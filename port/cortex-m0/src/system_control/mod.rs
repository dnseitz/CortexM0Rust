// system_control/mod.rs
// AltOSRust
//
// Created by Daniel Seitz on 11/30/16

use ::volatile::Volatile;
use ::peripheral::{Control, Register};

mod icsr;

pub fn scb() -> SCB {
  SCB::scb()
}

/// System Control Block
#[derive(Copy, Clone)]
pub struct SCB {
  mem_addr: usize,
  icsr: icsr::ICSR,
}

impl Control for SCB {
  unsafe fn mem_addr(&self) -> Volatile<usize> {
    Volatile::new(self.mem_addr as *const usize)
  }
}

impl SCB {
  fn scb() -> Self {
    const SCB_ADDR: usize = 0xE000_ED00;
    SCB {
      mem_addr: SCB_ADDR,
      icsr: icsr::ICSR::new(SCB_ADDR),
    }
  }

  pub fn set_pend_sv(&self) {
    self.icsr.set_pend_sv();
  }

  pub fn clear_pend_sv(&self) {
    self.icsr.clear_pend_sv();
  }
}
