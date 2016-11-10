#![cfg(test)]

use super::{Volatile, RawVolatile};

#[test]
fn store_volatile() {
  let num = 0xFF00;
  unsafe {
    let mut volatile = Volatile::new(&num);
    volatile.store(0xFF);
  }
  assert_eq!(num, 0xFF);
}

#[test]
fn load_volatile() {
  let num = 0xFF00;
  unsafe {
    let volatile = Volatile::new(&num);
    assert_eq!(num, volatile.load());
  }
}

#[test]
fn add_assign_volatile_deref() {
  let num = 0xFF00;
  unsafe {
    let mut volatile = Volatile::new(&num);
    *volatile += 0xFF;
  }
  assert_eq!(num, 0xFFFF);
}

#[test]
fn add_volatile_deref() {
  let num = 0xFF00;
  let num2 = unsafe {
    let volatile = Volatile::new(&num);
    *volatile + 0xFF
  };
  assert_eq!(num, 0xFF00);
  assert_eq!(num2, 0xFFFF);
}

#[test]
fn sub_assign_volatile_deref() {
  let num = 0xFF00;
  unsafe {
    let mut volatile = Volatile::new(&num);
    *volatile -= 0xFF00;
  }
  assert_eq!(num, 0x0000);
}

#[test]
fn sub_volatile_deref() {
  let num = 0xFF00;
  let num2 = unsafe {
    let volatile = Volatile::new(&num);
    *volatile - 0xFF00
  };
  assert_eq!(num, 0xFF00);
  assert_eq!(num2, 0x0000);
}

#[test]
fn mul_assign_volatile_deref() {
  let num = 10;
  unsafe {
    let mut volatile = Volatile::new(&num);
    *volatile *= 10;
  }
  assert_eq!(num, 100);
}

#[test]
fn mul_volatile_deref() {
  let num = 10;
  let num2 = unsafe {
    let volatile = Volatile::new(&num);
    *volatile * 10
  };
  assert_eq!(num, 10);
  assert_eq!(num2, 100);
}

#[test]
fn div_assign_volatile_deref() {
  let num = 100;
  unsafe {
    let mut volatile = Volatile::new(&num);
    *volatile /= 10;
  }
  assert_eq!(num, 10);
}

#[test]
fn div_volatile_deref() {
  let num = 100;
  let num2 = unsafe {
    let volatile = Volatile::new(&num);
    *volatile / 10
  };
  assert_eq!(num, 100);
  assert_eq!(num2, 10);
}

#[test]
fn bitand_assign_volatile_deref() {
  let num = 0xF0;
  unsafe {
    let mut volatile = Volatile::new(&num);
    *volatile &= 0xF;
  }
  assert_eq!(num, 0x00);
}

#[test]
fn bitand_volatile_deref() {
  let num = 0xF0;
  let num2 = unsafe {
    let volatile = Volatile::new(&num);
    *volatile & 0xF
  };
  assert_eq!(num, 0xF0);
  assert_eq!(num2, 0x00);
}

#[test]
fn bitor_assign_volatile_deref() {
  let num = 0xF0;
  unsafe {
    let mut volatile = Volatile::new(&num);
    *volatile |= 0xF;
  }
  assert_eq!(num, 0xFF);
}

#[test]
fn bitor_volatile_deref() {
  let num = 0xF0;
  let num2 = unsafe {
    let volatile = Volatile::new(&num);
    *volatile | 0xF
  };
  assert_eq!(num, 0xF0);
  assert_eq!(num2, 0xFF);
}
