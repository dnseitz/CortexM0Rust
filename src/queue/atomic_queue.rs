// queue/atomic_queue.rs
// AltOSRust
//
// Created by Daniel Seitz on 12/3/16
//! An atomic wrapper around the Queue struct. Able to be synchronized between threads.
use queue::{Queue, Node};
use alloc::boxed::Box;
use core::cell::UnsafeCell;

pub struct AtomicQueue<T> {
  internal: UnsafeCell<Queue<T>>,
}

unsafe impl<T: Send> Sync for AtomicQueue<T> {}
unsafe impl<T: Send> Send for AtomicQueue<T> {}

impl<T> AtomicQueue<T> {
  pub const fn new() -> Self {
    AtomicQueue { internal: UnsafeCell::new(Queue::new()) }
  }

  pub fn from(queue: Queue<T>) -> Self {
    AtomicQueue { internal: UnsafeCell::new(queue) }
  }

  pub fn enqueue(&self, elem: Box<Node<T>>) {
    atomic! {
      self.get_internal_mut().enqueue(elem);
    }
  }

  pub fn dequeue(&self) -> Option<Box<Node<T>>> {
    atomic! {
      self.get_internal_mut().dequeue()
    }
  }

  /*
  pub fn sorted_insert<F: Fn(&T, &T) -> bool>(&self, elem: Box<T>, sort: F) {
    atomic! {
      self.get_internal_mut().sorted_insert(elem, sort);
    }
  }
  */

  pub fn remove<F: Fn(&T) -> bool>(&self, predicate: F) -> Queue<T> {
    atomic! {
      self.get_internal_mut().remove(predicate)
    }
  }

  pub fn append(&self, to_append: Queue<T>) {
    atomic! {
      self.get_internal_mut().append(to_append);
    }
  }

  #[allow(deprecated)]
  pub fn modify_all<F: Fn(&mut T)>(&self, block: F) {
    atomic! {
      self.get_internal_mut().modify_all(block);
    }
  }

  pub fn remove_all(&self) -> Queue<T> {
    atomic! {
      self.get_internal_mut().remove_all()
    }
  }

  pub fn is_empty(&self) -> bool {
    atomic! {
      self.get_internal().is_empty()
    }
  }

  fn get_internal(&self) -> &Queue<T> {
    unsafe { &*self.internal.get() }
  }

  fn get_internal_mut(&self) -> &mut Queue<T> {
    unsafe { &mut *self.internal.get() }
  }
}
