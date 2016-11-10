
#[inline(always)]
pub unsafe fn dmb() {
  #[cfg(not(test))]
  asm!("dmb");
}

#[inline(always)]
pub unsafe fn bkpt() {
  #[cfg(not(test))]
  asm!("bkpt");
}
