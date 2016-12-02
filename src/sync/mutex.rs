// sync/mutex.rs
// AltOSRust
//
// Created by Daniel Seitz on 12/1/16

use atomic::Atomic;
use core::ops::{Drop, Deref, DerefMut};
use core::cell::UnsafeCell;

pub struct Mutex<T: ?Sized> {
  wchan: UnsafeCell<usize>,
  lock: Atomic<bool>,
  data: UnsafeCell<T>,
}

pub struct MutexGuard<'a, T: ?Sized + 'a> {
  wchan: &'a usize,
  lock: &'a Atomic<bool>,
  data: &'a mut T,
}

unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}
unsafe impl<T: ?Sized + Send> Send for Mutex<T> {}

impl<T> Mutex<T> {
  /// Creates a new mutex lock wrapping the supplied data
  pub const fn new(data: T) -> Self {
    Mutex {
      wchan: UnsafeCell::new(0),
      lock: Atomic::new(false),
      data: UnsafeCell::new(data),
    }
  }
}

impl<T: ?Sized> Mutex<T> {
  fn obtain_lock(&self) {
    while self.lock.compare_and_swap(false, true) != false {
      // let another process run if we can't get the lock
      let wchan = unsafe { &mut *self.wchan.get() };
      if *wchan == 0 {
        *wchan = chan::fetch_next_chan();
      }
      ::task::sleep(*wchan);
    }
  }

  /// Locks the mutex, blocking until it can hold the lock.
  pub fn lock(&self) -> MutexGuard<T> {
    self.obtain_lock();
    MutexGuard {
      wchan: unsafe { &*self.wchan.get() },
      lock: &self.lock,
      data: unsafe { &mut *self.data.get() },
    }
  }

  /// Tries to lock the mutex, if it's already locked then returns None. Otherwise it returns a
  /// guard in Some. This is a non-blocking operation.
  pub fn try_lock(&self) -> Option<MutexGuard<T>> {
    if self.lock.compare_and_swap(false, true) == false {
      Some(
        MutexGuard {
          wchan: unsafe { &*self.wchan.get() },
          lock: &self.lock,
          data: unsafe { &mut *self.data.get() },
        }
      )
    }
    else {
      None
    }
  }
}

impl<'a, T: ?Sized> Deref for MutexGuard<'a, T> {
  type Target = T;

  fn deref(&self) -> &T {
    &*self.data
  }
}

impl<'a, T: ?Sized> DerefMut for MutexGuard<'a, T> {
  fn deref_mut(&mut self) -> &mut T {
    &mut *self.data
  }
}

impl<'a, T: ?Sized> Drop for MutexGuard<'a, T> {
  /// Dropping the guard will unlock the lock it came from
  fn drop(&mut self) {
    self.lock.store(false);
    ::task::wake(*self.wchan);
  }
}

mod chan {
  use ::atomic::Atomic;

  static WCHAN: Atomic<usize> = Atomic::new(1);

  pub fn fetch_next_chan() -> usize {
    WCHAN.fetch_add(1)
  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn smoke() {
  }
}
