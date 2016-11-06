//! This module handles the clock control register of the CRR

use super::super::Register;
use super::Clock;

/// Clock Control Register
#[derive(Copy, Clone)]
pub struct ClockControl {
  cr: CR,
  cr2: CR2,
}

impl ClockControl {
  pub fn new(base_addr: u32) -> Self {
    ClockControl {
      cr: CR::new(base_addr),
      cr2: CR2::new(base_addr),
    }
  }

  pub fn enable_clock(&self, clock: Clock) {
    match clock {
      Clock::HSI | Clock::HSE | Clock::PLL => self.cr.set_clock(true, clock),
      Clock::HSI48 | Clock::HSI14 => self.cr2.set_clock(true, clock),
    };
  }

  pub fn disable_clock(&self, clock: Clock) -> bool {
    match clock {
      Clock::HSI | Clock::HSE | Clock::PLL => self.cr.set_clock(false, clock),
      Clock::HSI48 | Clock::HSI14 => self.cr2.set_clock(false, clock),
    }
  }

  pub fn clock_is_on(&self, clock: Clock) -> bool {
    match clock {
      Clock::HSI | Clock::HSE | Clock::PLL => self.cr.clock_is_on(clock),
      Clock::HSI48 | Clock::HSI14 => self.cr2.clock_is_on(clock),
    }
  }

  pub fn clock_is_ready(&self, clock: Clock) -> bool {
    match clock {
      Clock::HSI | Clock::HSE | Clock::PLL => self.cr.clock_is_ready(clock),
      Clock::HSI48 | Clock::HSI14 => self.cr2.clock_is_ready(clock),
    }
  }
}

#[derive(Copy, Clone)]
pub struct CR {
  base_addr: u32,
}

impl Register for CR {
  fn new(base_addr: u32) -> Self {
    CR { base_addr: base_addr }
  }

  fn mem_offset(&self) -> u32 {
    0x0
  }
}

impl CR {
  fn set_clock(&self, enable: bool, clock: Clock) -> bool {
    let mask = match clock {
      Clock::PLL => 1 << 24,
      Clock::HSE => 1 << 16,
      Clock::HSI => 1 << 0,
      _ => panic!("CR::enable_clock - argument clock is not controlled by this register!"),
    };
    
    unsafe {
      let reg = (self.base_addr + self.mem_offset()) as *mut u32;
      if enable {
        *reg |= mask;
        true
      }
      else {
        *reg &= !mask;
        (*reg & mask) == 0
      }
    }
  }

  fn clock_is_on(&self, clock: Clock) -> bool {
    let mask = match clock {
      Clock::PLL => 1 << 24,
      Clock::HSE => 1 << 16,
      Clock::HSI => 1 << 0,
      _ => panic!("CR::clock_is_on - argument clock is not controlled by thsi register!"),
    };

    unsafe {
      let reg = (self.base_addr + self.mem_offset()) as *mut u32;
      (*reg & mask) != 0
    }
  }

  fn clock_is_ready(&self, clock: Clock) -> bool {
    let mask = match clock {
      Clock::PLL => 1 << 25,
      Clock::HSE => 1 << 17,
      Clock::HSI => 1 << 1,
      _ => panic!("CR::clock_is_ready - argument clock is not controlled by this register!"),
    };

    unsafe {
      let reg = (self.base_addr + self.mem_offset()) as *mut u32;
      (*reg & mask) != 0
    }
  }
}

#[derive(Copy, Clone)]
pub struct CR2 {
  base_addr: u32,
}

impl Register for CR2 {
  fn new(base_addr: u32) -> Self {
    CR2 { base_addr: base_addr }
  }

  fn mem_offset(&self) -> u32 {
    0x34
  }
}

impl CR2 {
  fn set_clock(&self, enable: bool, clock: Clock) -> bool {
    let mask = match clock {
      Clock::HSI48 => 1 << 16,
      Clock::HSI14 => 1 << 0,
      _ => panic!("CR2::set_clock - argument clock is not controlled by this register!"),
    };

    unsafe {
      let reg = (self.base_addr + self.mem_offset()) as *mut u32;
      if enable {
        *reg |= mask;
        true
      }
      else {
        *reg &= !mask;
        (*reg & mask) == 0
      }
    }
  }

  fn clock_is_on(&self, clock: Clock) -> bool {
    let mask = match clock {
      Clock::HSI48 => 1 << 16,
      Clock::HSI14 => 1 << 0,
      _ => panic!("CR2::clock_is_on - argument clock is not controlled by this register!"),
    };

    unsafe {
      let reg = (self.base_addr + self.mem_offset()) as *mut u32;
      (*reg & mask) != 0
    }
  }

  fn clock_is_ready(&self, clock: Clock) -> bool {
    let mask = match clock {
      Clock::HSI48 => 1 << 17,
      Clock::HSI14 => 1 << 1,
      _ => panic!("CR2::clock_is_ready - argument clock is not controlled by this register!"),
    };

    unsafe {
      let reg = (self.base_addr + self.mem_offset()) as *mut u32;
      (*reg & mask) != 0
    }
  }
}
