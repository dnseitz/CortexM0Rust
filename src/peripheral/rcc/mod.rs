//! This module controls the RCC (Reset and Clock Controller), it handles setting the clock values
//! and the reset flags that are set on a reset.

use super::{Peripheral, Register};

mod clock_control;
mod config;

pub fn rcc() -> RCC {
  RCC::rcc()
}

pub enum Clock {
  HSI,
  HSI48,
  HSI14,
  HSE,
  PLL,
}

/// Reset and Clock Controller
#[derive(Copy, Clone)]
pub struct RCC {
  mem_addr: u32,
  cr: clock_control::ClockControl,
  cfgr: config::CFGR,
}

impl Peripheral for RCC {
  fn mem_addr(&self) -> u32 {
    self.mem_addr
  }
}

impl RCC {
  fn rcc() -> Self {
    const RCC_ADDR: u32 = 0x4002_1000;
    RCC {
      mem_addr: RCC_ADDR,
      cr: clock_control::ClockControl::new(RCC_ADDR),
      cfgr: config::CFGR::new(RCC_ADDR),
    }
  }

  pub fn enable_clock(&self, clock: Clock) {
    self.cr.enable_clock(clock);
  }

  pub fn disable_clock(&self, clock: Clock) -> bool {
    self.cr.disable_clock(clock)
  }

  pub fn clock_is_on(&self, clock: Clock) -> bool {
    self.cr.clock_is_on(clock)
  }

  pub fn clock_is_ready(&self, clock: Clock) -> bool {
    self.cr.clock_is_ready(clock)
  }

  pub fn get_system_clock_source(&self) -> Clock {
    self.cfgr.get_system_clock_source()
  }

  pub fn set_system_clock_source(&self, clock: Clock) {
    self.cfgr.set_system_clock_source(clock);
  }

  pub fn get_pll_source(&self) -> Clock {
    self.cfgr.get_pll_source()
  }

  pub fn set_pll_source(&self, clock: Clock) {
    self.cfgr.set_pll_source(clock);
  }

  pub fn get_pll_multiplier(&self) -> u8 {
    self.cfgr.get_pll_multiplier()
  }

  pub fn set_pll_multiplier(&self, mul: u8) {
    self.cfgr.set_pll_multiplier(mul);
  }
}

