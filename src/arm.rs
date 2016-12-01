// arm.rs
// AltOSRust
//
// Created by Daniel Seitz on 11/30/16

pub use self::imp::*;

#[cfg(target_arch="arm")]
mod imp {
  #[inline(always)]
  pub unsafe fn dmb() {
    asm!("dmb");
  }

  #[inline(always)]
  pub unsafe fn bkpt() {
    asm!("bkpt");
  }

  #[inline(always)]
  pub unsafe fn enable_interrupts() {
    asm!("cpsie i");
  }

  #[inline(always)]
  pub unsafe fn disable_interrupts() {
    asm!("cpsid i");
  }
}

#[cfg(not(target_arch="arm"))]
mod imp {
  #[inline(always)]
  pub unsafe fn dmb() {}

  #[inline(always)]
  pub unsafe fn bkpt() {}

  #[inline(always)]
  pub unsafe fn enable_interrupts() {}

  #[inline(always)]
  pub unsafe fn disable_interrupts() {}
}
