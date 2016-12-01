
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

unsafe impl<T> Sync for Atomic<T> {}

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
