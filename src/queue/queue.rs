// queue/queue.rs
// AltOSRust
//
// Created by Daniel Seitz on 12/2/16

use task::TaskControl;
use alloc::boxed::Box;

pub trait Queuable {
  fn set_next(&mut self, Option<Box<Self>>);
  fn take_next(&mut self) -> Option<Box<Self>>;
}

pub struct Queue<T: Queuable> {
  head: Option<Box<T>>,
  tail: *mut T,
}

impl<T: Queuable> Queue<T> {
  pub const fn new() -> Self {
    Queue { 
      head: None,
      tail: ::core::ptr::null_mut(),
    }
  }

  /// Place a new item onto the end of the queue.
  /// O(1) algorithmic time
  pub fn enqueue(&mut self, elem: Box<T>) {
    let mut new_tail = elem;

    let raw_tail: *mut _ = &mut *new_tail;

    if !self.tail.is_null() {
      unsafe {
        (*self.tail).set_next(Some(new_tail));
      }
    }
    else {
      self.head = Some(new_tail);
    }

    self.tail = raw_tail;
  }

  /// Take an item off of the front of the queue. If there are no items in the queue returns None.
  /// O(1) algorithmic time
  pub fn dequeue(&mut self) -> Option<Box<T>> {
    match self.head.take() {
      Some(mut head) => {
        self.head = head.take_next();

        if self.head.is_none() {
          self.tail = ::core::ptr::null_mut();
        }

        Some(head)
      },
      None => None,
    }
  }
  
  /// Remove all elements matching `predicate` and return them in a new queue
  /// O(n) algorithmic time
  pub fn remove<F: Fn(&T) -> bool>(&mut self, predicate: F) -> Queue<T> {
    let mut matching = Queue::new();
    let old_tail = self.tail;

    while let Some(mut head) = self.head.take() {
      self.head = head.take_next();
      if self.head.is_none() {
        self.tail = ::core::ptr::null_mut();
      }

      if predicate(&head) {
        matching.enqueue(head);
      }
      else {
        self.enqueue(head);
      }
      if let Some(head) = self.head.as_mut() {
        if &mut **head as *mut _ == old_tail {
          break;
        }
      }
    }
    matching
  }
  
  /// Append all the elements of `queue` onto self.
  /// O(1) algorithmic time
  pub fn append(&mut self, mut queue: Queue<T>) {
    if !self.tail.is_null() {
      unsafe {
        (*self.tail).set_next(queue.head.take());
      }
    }
    else {
      self.head = queue.head.take();
    }

    self.tail = queue.tail;
  }

  /// Modify all the elements of the queue with the block passed in.
  /// O(n) algorithmic time
  pub fn modify_all<F: Fn(&mut T)>(&mut self, block: F) {
    let mut scratch = Queue::new();
    while let Some(mut node) = self.dequeue() {
      block(&mut *node);
      scratch.enqueue(node);
    }
    *self = scratch;
  }

  pub fn remove_all(&mut self) -> Queue<T> {
    ::core::mem::replace(self, Queue::new())
  }
}

pub struct Node<T> {
  data: T,
  next: Option<Box<Node<T>>>,
}

impl<T> Node<T> {
  pub fn new(data: T) -> Self {
    Node { 
      data: data,
      next: None,
    }
  }
}

impl<T> Queuable for Node<T> {
  fn take_next(&mut self) -> Option<Box<Self>> {
    self.next.take()
  }

  fn set_next(&mut self, next: Option<Box<Self>>) {
    self.next = next;
  }
}

#[cfg(test)]
mod tests {
  use super::{Queue, Node};
  use alloc::boxed::Box;

  #[test]
  fn empty_dequeue() {
    let mut list: Queue<Node<usize>> = Queue::new();

    assert!(list.dequeue().is_none());
  }

  #[test]
  fn basics() {
    let mut list = Queue::new();

    // Populate list
    list.enqueue(Box::new(Node::new(1)));
    list.enqueue(Box::new(Node::new(2)));
    list.enqueue(Box::new(Node::new(3)));

    // Check normal removal
    assert_eq!(list.dequeue().unwrap().data, 1);
    assert_eq!(list.dequeue().unwrap().data, 2);

    // Push some more just to make sure nothing's corrupted
    list.enqueue(Box::new(Node::new(4)));
    list.enqueue(Box::new(Node::new(5)));

    // Check normal removal
    assert_eq!(list.dequeue().unwrap().data, 3);
    assert_eq!(list.dequeue().unwrap().data, 4);

    // Check exhaustion
    assert_eq!(list.dequeue().unwrap().data, 5);
    assert!(list.dequeue().is_none());

    // Check the exhaustion case fixed the pointer right
    list.enqueue(Box::new(Node::new(6)));
    list.enqueue(Box::new(Node::new(7)));

    // Check normal removal
    assert_eq!(list.dequeue().unwrap().data, 6);
    assert_eq!(list.dequeue().unwrap().data, 7);
    assert!(list.dequeue().is_none());
  }

  #[test]
  fn remove_predicate() {
    let mut list = Queue::new();

    list.enqueue(Box::new(Node::new(1)));
    list.enqueue(Box::new(Node::new(2)));
    list.enqueue(Box::new(Node::new(3)));
    list.enqueue(Box::new(Node::new(1)));
    list.enqueue(Box::new(Node::new(2)));
    list.enqueue(Box::new(Node::new(3)));

    let predicate = |task: &Node<usize>| task.data == 1;

    let mut removed = list.remove(predicate);
    assert_eq!(removed.dequeue().unwrap().data, 1);
    assert_eq!(removed.dequeue().unwrap().data, 1);
    assert!(removed.dequeue().is_none());
  }

  #[test]
  fn append_queue() {
    let mut list1 = Queue::new();
    let mut list2 = Queue::new();

    list1.enqueue(Box::new(Node::new(1)));
    list1.enqueue(Box::new(Node::new(2)));
    list2.enqueue(Box::new(Node::new(3)));
    list2.enqueue(Box::new(Node::new(4)));

    list1.append(list2);

    assert_eq!(list1.dequeue().unwrap().data, 1);
    assert_eq!(list1.dequeue().unwrap().data, 2);
    assert_eq!(list1.dequeue().unwrap().data, 3);
    assert_eq!(list1.dequeue().unwrap().data, 4);

    assert!(list1.dequeue().is_none());
  }

  #[test]
  fn append_empty() {
    let mut list1 = Queue::new();
    let mut list2 = Queue::new();

    list1.enqueue(Box::new(Node::new(1)));
    list1.enqueue(Box::new(Node::new(2)));

    list1.append(list2);

    assert_eq!(list1.dequeue().unwrap().data, 1);
    assert_eq!(list1.dequeue().unwrap().data, 2);

    assert!(list1.dequeue().is_none());
  }

  #[test]
  fn modify_all() {
    let mut list = Queue::new();

    list.enqueue(Box::new(Node::new(1)));
    list.enqueue(Box::new(Node::new(2)));
    list.enqueue(Box::new(Node::new(3)));

    list.modify_all(|task: &mut Node<usize>| task.data *= 10);

    assert_eq!(list.dequeue().unwrap().data, 10);
    assert_eq!(list.dequeue().unwrap().data, 20);
    assert_eq!(list.dequeue().unwrap().data, 30);

    assert!(list.dequeue().is_none());
  }

  #[test]
  fn remove_all() {
    let mut list = Queue::new();

    list.enqueue(Box::new(Node::new(1)));
    list.enqueue(Box::new(Node::new(2)));
    list.enqueue(Box::new(Node::new(3)));

    let mut old = list.remove_all();

    assert_eq!(old.dequeue().unwrap().data, 1);
    assert_eq!(old.dequeue().unwrap().data, 2);
    assert_eq!(old.dequeue().unwrap().data, 3);
    assert!(old.dequeue().is_none());

    assert!(list.dequeue().is_none());

  }
}
