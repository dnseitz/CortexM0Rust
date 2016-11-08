//! This module controls the RCC (Reset and Clock Controller), it handles enabling and disabling
//! clocks, setting clock configurations and the reset flags that are set on a reset.

use super::{Peripheral, Register};

mod clock_control;
mod config;

mod clock_rate {
  static mut clock_rate: u32 = 0;

  pub fn get_system_clock_rate() -> u32 {
    unsafe { 
      clock_rate 
    }
  }

  pub fn update_system_clock_rate() {
    const HSI_VALUE: u32 = 8_000_000;
    const HSE_VALUE: u32 = 8_000_000;
    const HSI48_VALUE: u32 = 48_000_000;
    use super::Clock;

    let rcc = super::rcc();
    let rate = match rcc.get_system_clock_source() {
      Clock::HSI => HSI_VALUE,
      Clock::HSE => HSE_VALUE,
      Clock::HSI48 => HSI48_VALUE,
      Clock::PLL => {
        let multiplier = rcc.get_pll_multiplier() as u32;
        let source = rcc.get_pll_source();
        let prediv_factor = rcc.get_pll_prediv_factor() as u32;

        match source {
          Clock::HSE => (HSE_VALUE/prediv_factor) * multiplier,
          Clock::HSI48 => (HSI48_VALUE/prediv_factor) * multiplier,
          Clock::HSI => (HSI_VALUE/2) * multiplier,
          _ => panic!("CRR::update_system_core_clock - invalid clock driving the PLL!"),
        }
      },
      _ => panic!("CRR::update_system_core_clock - invalid clock for the system clock!"),
    };

    unsafe { clock_rate = rate; }
  }

}

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
  cfgr: config::ConfigControl,
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
      cfgr: config::ConfigControl::new(RCC_ADDR),
    }
  }

  /// Enable the specified clock
  pub fn enable_clock(&self, clock: Clock) {
    self.cr.enable_clock(clock);
  }

  /// Disable the specified clock, if the clock cannot be disabled (if it is driving the PLL for
  /// example) then this method will return false, it returns true otherwise
  pub fn disable_clock(&self, clock: Clock) -> bool {
    self.cr.disable_clock(clock)
  }

  /// Return true if the specified clock is enabled
  pub fn clock_is_on(&self, clock: Clock) -> bool {
    self.cr.clock_is_on(clock)
  }

  /// Return true if the specified clock is ready to be used as the system clock
  pub fn clock_is_ready(&self, clock: Clock) -> bool {
    self.cr.clock_is_ready(clock)
  }

  /// Return the clock driving the system clock
  pub fn get_system_clock_source(&self) -> Clock {
    self.cfgr.get_system_clock_source()
  }

  /// Set the system clock source. The system clock can only be run off of the HSI, HSE, PLL or
  /// HSI48 clocks, if another clock is specified the kernel will panic
  pub fn set_system_clock_source(&self, clock: Clock) {
    self.cfgr.set_system_clock_source(clock);
    clock_rate::update_system_clock_rate();
  }

  /// Get the clock driving the PLL
  pub fn get_pll_source(&self) -> Clock {
    self.cfgr.get_pll_source()
  }

  /// Set the specified clock to drive the PLL, only the HSI, HSE or HSI48 can drive the PLL, if
  /// another clock is specified the kernel will panic.
  pub fn set_pll_source(&self, clock: Clock) {
    self.cfgr.set_pll_source(clock);
  }

  
  /// Get the current multiplier for the PLL, the multiplier is in a range of [2..16]. 
  pub fn get_pll_multiplier(&self) -> u8 {
    self.cfgr.get_pll_multiplier()
  }

  /// Set the PLL multiplier, the multiplier specified MUST be within the range of [2..16], if it
  /// is outside of that range the kernel will panic.
  pub fn set_pll_multiplier(&self, mul: u8) {
    self.cfgr.set_pll_multiplier(mul);
  }

  /// Get the current prediv factor for the PLL, the factor is in a range of [1..16].
  pub fn get_pll_prediv_factor(&self) -> u8 {
    self.cfgr.get_pll_prediv_factor()
  }

  /// Set the PLL prediv factor, the factor specified MUST be within the range of [1..16], if it is
  /// outside that range the kernel will panic.
  pub fn set_pll_prediv_factor(&self, factor: u8) {
    self.cfgr.set_pll_prediv_factor(factor);
  }

  pub fn get_system_clock_rate(&self) -> u32 {
    clock_rate::get_system_clock_rate()
  }
}

