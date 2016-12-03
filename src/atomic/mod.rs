// atomic/mod.rs
// AltOSRust
//
// Created by Daniel Seitz on 11/30/16

use ::core::cell::UnsafeCell;
use ::core::ops::{Add, Sub, BitAnd, BitOr, BitXor};

macro_rules! start_critical {
  ($var:ident) => {{
    unsafe {
      #![cfg(target_arch="arm")]
      asm!(
        concat!(
          "mrs $0, PRIMASK\n",
          "cpsid i\n")
        : "=r"($var)
        : /* no inputs */
        : /* no clobbers */
        : "volatile");
    }
  }}
}

macro_rules! end_critical {
  ($var:ident) => {{
    unsafe {
      #![cfg(target_arch="arm")]
      asm!("msr PRIMASK, $0"
        : /* no outputs */
        : "r"($var)
        : /* no clobbers */
        : "volatile");
    }
  }}
}

macro_rules! atomic {
  { $( $code:expr );*; } => {{
    let primask: u32;
    start_critical!(primask);
    $(
      $code;
    )*
    end_critical!(primask);
  }};
  { $( $code:expr );*; $last:expr } => {{
    let primask: u32;
    start_critical!(primask);
    $(
      $code;
    )*
    let result = $last;
    end_critical!(primask);
    result
  }};
  { $last:expr } => {{
    let primask: u32;
    start_critical!(primask);
    let result = $last;
    end_critical!(primask);
    result
  }}
}

pub struct Atomic<T> {
  data: UnsafeCell<T>,
}

unsafe impl<T: Send> Sync for Atomic<T> {}

impl<T: Copy> Atomic<T> {
  pub const fn new(data: T) -> Self {
    Atomic { data: UnsafeCell::new(data) }
  }

  pub fn load(&self) -> T {
    atomic! {
      unsafe {
        *self.data.get()
      }
    }
  }

  pub fn store(&self, data: T) {
    atomic! {
      unsafe {
        *self.data.get() = data;
      }
    }
  }

  pub fn swap(&self, data: T) -> T {
    atomic! {
      unsafe {
        ::core::mem::replace(&mut *self.data.get(), data)
      }
    }
  }
}

impl<T: Copy + Default> Default for Atomic<T> {
  fn default() -> Self {
    Atomic { data: UnsafeCell::new(T::default()) }
  }
}

impl<T: Copy + PartialOrd> Atomic<T> {
  pub fn compare_and_swap(&self, current: T, new: T) -> T {
    match self.compare_exchange(current, new) {
      Ok(x) => x,
      Err(x) => x,
    }
  }

  pub fn compare_exchange(&self, current: T, new: T) -> Result<T, T> {
    atomic! {
      unsafe {
        let old = self.data.get();
        if *old == current {
          Ok(::core::mem::replace(&mut *old, new))
        }
        else {
          Err(*old)
        }
      }
    }
  }
}

impl<T: Copy + Add<Output=T>> Atomic<T> {
  pub fn fetch_add(&self, data: T) -> T {
    atomic! {
      unsafe {
        let old = self.data.get();
        let new = *old + data;
        ::core::mem::replace(&mut *old, new)
      }
    }
  }
}

impl<T: Copy + Sub<Output=T>> Atomic<T> {
  pub fn fetch_sub(&self, data: T) -> T {
    atomic! {
      unsafe {
        let old = self.data.get();
        let new = *old - data;
        ::core::mem::replace(&mut *old, new)
      }
    }
  }
}

impl<T: Copy + BitAnd<Output=T>> Atomic<T> {
  pub fn fetch_and(&self, data: T) -> T {
    atomic! {
      unsafe {
        let old = self.data.get();
        let new = *old & data;
        ::core::mem::replace(&mut *old, new)
      }
    }
  }

}

impl<T: Copy + BitOr<Output=T>> Atomic<T> {
  pub fn fetch_or(&self, data: T) -> T {
    atomic! {
      unsafe {
        let old = self.data.get();
        let new = *old | data;
        ::core::mem::replace(&mut *old, new)
      }
    }
  }

}

impl<T: Copy + BitXor<Output=T>> Atomic<T> {
  pub fn fetch_xor(&self, data: T) -> T {
    atomic! {
      unsafe {
        let old = self.data.get();
        let new = *old ^ data;
        ::core::mem::replace(&mut *old, new)
      }
    }
  }
}

#[cfg(test)]
mod tests {
  // As a side note, these operations are not actually atomic when compiled 
  // for anything other than ARM
  use super::Atomic;

  #[test]
  fn load() {
    let atom: Atomic<usize> = Atomic::new(0);

    assert_eq!(atom.load(), 0);
  }

  #[test]
  fn store() {
    let atom: Atomic<usize> = Atomic::new(0);

    atom.store(1);
    assert_eq!(atom.load(), 1);
  }

  #[test]
  fn swap() {
    let atom: Atomic<usize> = Atomic::new(0);

    assert_eq!(atom.swap(1), 0);
    assert_eq!(atom.load(), 1);
  }

  #[test]
  fn compare_and_swap() {
    let atom: Atomic<usize> = Atomic::new(0);

    assert_eq!(atom.compare_and_swap(0, 1), 0);
    assert_eq!(atom.load(), 1);

  }

  #[test]
  fn compare_and_swap_fail() {
    let atom: Atomic<usize> = Atomic::new(0);

    assert_eq!(atom.compare_and_swap(1, 2), 0);
    assert_eq!(atom.load(), 0);
  }

  #[test]
  fn compare_exchange() {
    let atom: Atomic<usize> = Atomic::new(0);

    assert_eq!(atom.compare_exchange(0, 1), Ok(0));
    assert_eq!(atom.load(), 1);
  }

  #[test]
  fn compare_exchange_fail() {
    let atom: Atomic<usize> = Atomic::new(0);

    assert_eq!(atom.compare_exchange(1, 2), Err(0));
    assert_eq!(atom.load(), 0);
  }

  #[test]
  fn fetch_add() {
    let atom: Atomic<usize> = Atomic::new(0);

    assert_eq!(atom.fetch_add(1), 0);
    assert_eq!(atom.fetch_add(1), 1);
    assert_eq!(atom.load(), 2);
  }

  #[test]
  fn fetch_sub() {
    let atom: Atomic<usize> = Atomic::new(10);

    assert_eq!(atom.fetch_sub(1), 10);
    assert_eq!(atom.fetch_sub(4), 9);
    assert_eq!(atom.load(), 5);
  }

  #[test]
  fn fetch_and() {
    let atom: Atomic<usize> = Atomic::new(0xFF);

    assert_eq!(atom.fetch_and(0xAA), 0xFF);
    assert_eq!(atom.fetch_and(0xF), 0xAA);
    assert_eq!(atom.load(), 0xA);
  }

  #[test]
  fn fetch_or() {
    let atom: Atomic<usize> = Atomic::new(0);

    assert_eq!(atom.fetch_or(0xAA), 0x0);
    assert_eq!(atom.fetch_or(0x55), 0xAA);
    assert_eq!(atom.load(), 0xFF);
  }

  #[test]
  fn fetch_xor() {
    let atom: Atomic<usize> = Atomic::new(0xAA);

    assert_eq!(atom.fetch_xor(0xFF), 0xAA);
    assert_eq!(atom.fetch_xor(0xFF), 0x55);
    assert_eq!(atom.load(), 0xAA);
  }
}
