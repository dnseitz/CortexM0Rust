// sync/spin_mutex.rs
// AltOSRust
//
// Created by Daniel Seitz on 12/8/16

use atomic::{ATOMIC_BOOL_INIT, AtomicBool, Ordering};
use core::ops::{Drop, Deref, DerefMut};
use core::cell::UnsafeCell;

pub struct SpinMutex<T: ?Sized> {
  lock: AtomicBool,
  data: UnsafeCell<T>,
}

pub struct MutexGuard<'mx, T: ?Sized + 'mx> {
  lock: &'mx AtomicBool,
  data: &'mx mut T,
}

unsafe impl<T: ?Sized + Send> Send for SpinMutex<T> {}
unsafe impl<T: ?Sized + Send> Sync for SpinMutex<T> {}

impl<T> SpinMutex<T> {
  pub const fn new(data: T) -> Self {
    SpinMutex {
      lock: ATOMIC_BOOL_INIT,
      data: UnsafeCell::new(data),
    }
  }
}

impl<T: ?Sized> SpinMutex<T> {
  fn obtain_lock(&self) {
    while self.lock.compare_and_swap(false, true, Ordering::Acquire) != false {/* spin */}
  }

  pub fn lock(&self) -> MutexGuard<T> {
    self.obtain_lock();
    MutexGuard {
      lock: &self.lock,
      data: unsafe { &mut *self.data.get() },
    }
  }

  pub fn try_lock(&self) -> Option<MutexGuard<T>> {
    if self.lock.compare_and_swap(false, true, Ordering::Acquire) == false {
      Some(
        MutexGuard {
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
    self.lock.store(false, Ordering::Release);
  }
}
