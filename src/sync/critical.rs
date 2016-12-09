// sync/critical.rs
// AltOSRust
//
// Created by Daniel Seitz on 12/8/16

use core::ops::Drop;

// Provided by the portability layer to ensure that interrupts are disabled for a critical section
extern {
  fn begin_critical() -> usize;
  fn end_critical(mask: usize);
}

pub struct CriticalSection;

impl CriticalSection {
  pub fn begin() -> CriticalSectionGuard {
    //unsafe { CriticalSectionGuard(begin_critical()) }
    CriticalSectionGuard(0)
  }
}

pub struct CriticalSectionGuard(usize);

impl Drop for CriticalSectionGuard {
  fn drop(&mut self) {
    //unsafe { end_critical(self.0) };
  }
}

