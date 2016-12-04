// math.rs
// AltOSRust
//
// Created by Daniel Seitz on 11/30/16

#[no_mangle]
pub unsafe extern fn __aeabi_memclr4(dest: *mut u32, mut n: usize) {
  while n > 0 {
    *dest = 0;
    n -= 1;
  }
}

#[no_mangle]
pub unsafe extern fn __aeabi_lmul(num1: u64, num2: u64) -> u64 {
  let mut res = 0;
  let (higher, mut lower) = if num1 > num2 { (num1, num2) } else { (num2, num1) };
  // Incredibly unoptimized...
  while lower > 0 {
    res += higher;
    lower -= 1;
  }
  res
}

#[no_mangle]
pub unsafe extern fn __aeabi_uidiv(num: u32, den: u32) -> u32 {
  __aeabi_uidivbase(num, den, false) 
}

#[no_mangle]
pub unsafe extern fn __aeabi_uidivmod(num: u32, den: u32) -> u32 {
  __aeabi_uidivbase(num, den, true)
}

fn __aeabi_uidivbase(mut num: u32, mut den: u32, modwanted: bool) -> u32 {
  let mut bit: u32 = 1;
  let mut res: u32 = 0;

  while den < num && bit != 0 && (den & (1<<31)) == 0 {
    den <<= 1;
    bit <<= 1;
  }
  while bit != 0 {
    if num >= den {
      num -= den;
      res |= bit;
    }
    bit >>= 1;
    den >>= 1;
  }
  if modwanted { num } else { res }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn divide_even() {
    unsafe {
      assert_eq!(10, __aeabi_uidiv(100, 10));
    }
  }

  #[test]
  fn divide_uneven() {
    unsafe {
      assert_eq!(10, __aeabi_uidiv(105, 10));
    }
  }

  #[test]
  fn divide_denominator_bigger() {
    unsafe {
      assert_eq!(0, __aeabi_uidiv(5, 10));
    }
  }

  #[test]
  fn mod_even() {
    unsafe {
      assert_eq!(0, __aeabi_uidivmod(100, 10));
    }
  }
  
  #[test]
  fn mod_uneven() {
    unsafe {
      assert_eq!(5, __aeabi_uidivmod(105, 10));
    }
  }
  
  #[test]
  fn mod_denominator_bigger() {
    unsafe {
      assert_eq!(5, __aeabi_uidivmod(5, 10));
    }
  }

  #[test]
  fn multiply_bigger_first() {
    unsafe {
      assert_eq!(100, __aeabi_lmul(20, 5));
    }
  }

  #[test]
  fn multiply_bigger_second() {
    unsafe {
      assert_eq!(100, __aeabi_lmul(5, 20));
    }
  }
}
