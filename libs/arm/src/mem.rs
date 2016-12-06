// mem.rs
// AltOSRust
//
// Created by Daniel Seitz on 12/6/16

#[no_mangle]
pub unsafe extern fn __aeabi_memclr4(dest: *mut u32, mut n: isize) {
  while n > 0 {
    n -= 1;
    *dest.offset(n) = 0;
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn memclr() {
    let mut block: [u32; 10] = [0xAAAAAAAA; 10];
    
    for i in 0..10 {
      assert_eq!(block[i], 0xAAAAAAAA);
    }
    
    unsafe { __aeabi_memclr4(block.as_mut_ptr(), 10) };

    for i in 0..10 {
      assert_eq!(block[i], 0x0);
    }
  }
}
