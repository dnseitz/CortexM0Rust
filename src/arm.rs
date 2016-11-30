#![cfg(not(test))]

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
