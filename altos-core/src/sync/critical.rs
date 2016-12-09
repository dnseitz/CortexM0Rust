// sync/critical.rs
// AltOSRust
//
// Created by Daniel Seitz on 12/8/16

use core::ops::Drop;

pub struct CriticalSection;

impl CriticalSection {
  pub fn begin() -> CriticalSectionGuard {
    unsafe { CriticalSectionGuard(::begin_critical()) }
  }
}

pub struct CriticalSectionGuard(usize);

impl Drop for CriticalSectionGuard {
  fn drop(&mut self) {
    unsafe { ::end_critical(self.0) };
  }
}

