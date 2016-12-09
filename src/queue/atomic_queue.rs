// queue/atomic_queue.rs
// AltOSRust
//
// Created by Daniel Seitz on 12/3/16
//! An atomic wrapper around the Queue struct. Able to be synchronized between threads.
use queue::{Queue, Node};
use alloc::boxed::Box;
use sync::spin::{SpinMutex, MutexGuard};

pub struct AtomicQueue<T> {
  lock: SpinMutex<Queue<T>>,
}

unsafe impl<T: Send> Sync for AtomicQueue<T> {}
unsafe impl<T: Send> Send for AtomicQueue<T> {}

impl<T> AtomicQueue<T> {
  pub const fn new() -> Self {
    AtomicQueue { lock: SpinMutex::new(Queue::new()) }
  }

  pub fn from(queue: Queue<T>) -> Self {
    AtomicQueue { lock: SpinMutex::new(queue) }
  }

  pub fn enqueue(&self, elem: Box<Node<T>>) {
    let mut queue = self.lock();
    queue.enqueue(elem);
  }

  pub fn dequeue(&self) -> Option<Box<Node<T>>> {
    let mut queue = self.lock();
    queue.dequeue()
  }

  pub fn remove<F: Fn(&T) -> bool>(&self, predicate: F) -> Queue<T> {
    let mut queue = self.lock();
    queue.remove(predicate)
  }

  pub fn append(&self, to_append: Queue<T>) {
    let mut queue = self.lock();
    queue.append(to_append);
  }

  #[allow(deprecated)]
  pub fn modify_all<F: Fn(&mut T)>(&self, block: F) {
    let mut queue = self.lock();
    queue.modify_all(block);
  }

  pub fn remove_all(&self) -> Queue<T> {
    let mut queue = self.lock();
    queue.remove_all()
  }

  pub fn is_empty(&self) -> bool {
    let queue = self.lock();
    queue.is_empty()
  }

  fn lock(&self) -> MutexGuard<Queue<T>> {
    self.lock.lock()
  }
}

impl<T> Default for AtomicQueue<T> {
  fn default() -> Self {
    AtomicQueue::new()
  }
}
