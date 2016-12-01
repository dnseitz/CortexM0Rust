
#[inline(always)]
pub unsafe fn dmb() {
  #[cfg(target_arch="arm")]
  asm!("dmb");
}

#[inline(always)]
pub unsafe fn bkpt() {
  #[cfg(target_arch="arm")]
  asm!("bkpt");
}

#[inline(always)]
pub unsafe fn enable_interrupts() {
  #[cfg(target_arch="arm")]
  asm!("cpsie i");
}

#[inline(always)]
pub unsafe fn disable_interrupts() {
  #[cfg(target_arch="arm")]
  asm!("cpsid i");
}
