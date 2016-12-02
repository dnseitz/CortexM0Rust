// sync/mutex.rs
// AltOSRust
//
// Created by Daniel Seitz on 12/1/16

use atomic::Atomic;
use core::ops::{Drop, Deref, DerefMut};
use core::cell::UnsafeCell;

pub struct Mutex<T: ?Sized> {
  lock: Atomic<bool>,
  data: UnsafeCell<T>,
}

pub struct MutexGuard<'mx, T: ?Sized + 'mx> {
  wchan: usize,
  lock: &'mx Atomic<bool>,
  data: &'mx mut T,
}

unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}
unsafe impl<T: ?Sized + Send> Send for Mutex<T> {}

impl<T> Mutex<T> {
  /// Creates a new mutex lock wrapping the supplied data
  pub const fn new(data: T) -> Self {
    Mutex {
      lock: Atomic::new(false),
      data: UnsafeCell::new(data),
    }
  }
}

impl<T: ?Sized> Mutex<T> {
  fn wchan(&self) -> usize {
    &self.lock as *const _ as usize
  }

  fn obtain_lock(&self) {
    while self.lock.compare_and_swap(false, true) != false {
      // let another process run if we can't get the lock
      let wchan = self.wchan();
      ::task::sleep(wchan);
    }
  }

  /// Locks the mutex, blocking until it can hold the lock.
  pub fn lock(&self) -> MutexGuard<T> {
    self.obtain_lock();
    MutexGuard {
      wchan: self.wchan(),
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
          wchan: self.wchan(),
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

impl<'mx, T: ?Sized> Deref for MutexGuard<'mx, T> {
  type Target = T;

  fn deref(&self) -> &T {
    &*self.data
  }
}

impl<'mx, T: ?Sized> DerefMut for MutexGuard<'mx, T> {
  fn deref_mut(&mut self) -> &mut T {
    &mut *self.data
  }
}

impl<'mx, T: ?Sized> Drop for MutexGuard<'mx, T> {
  /// Dropping the guard will unlock the lock it came from and wake any tasks waiting on it.
  fn drop(&mut self) {
    // Do we care if we get pre-empted and another thread steals the lock before we wake the
    // sleeping tasks?
    atomic! {
      self.lock.store(false);
      ::task::wake(self.wchan);
    }
  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn smoke() {
  }
}
