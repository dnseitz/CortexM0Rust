// queue/sorted_list.rs
// AltOSRust
//
// Created by Daniel Seitz on 12/4/16

use super::Queueable;
use alloc::boxed::Box;

pub struct SortedList<T: Queueable + PartialOrd> {
  head: Option<Box<T>>,
}

impl<T: Queueable + PartialOrd> SortedList<T> {
  pub const fn new() -> Self {
    SortedList {
      head: None,
    }
  }

  /// Place a new item onto the end of the queue.
  /// O(1) algorithmic time
  pub fn insert(&mut self, mut elem: Box<T>) {
    if self.head.is_none() || *elem <= **self.head.as_ref().unwrap() {
      elem.set_next(self.head.take());
      self.head = Some(elem);
      return;
    }
    let mut current = self.head.as_mut();
    while let Some(node) = current.take() {
      if node.next().is_none() || *elem <= **node.next().unwrap() {
        current = Some(node);
        break;
      }
      current = node.next_mut();
    }

    if let Some(node) = current.take() {
      if node.next().is_none() {
      }
      elem.set_next(node.take_next());
      node.set_next(Some(elem));
    }
  }

  /// Take an item off of the front of the list. If there are no items in the list returns None.
  /// O(1) algorithmic time
  pub fn pop(&mut self) -> Option<Box<T>> {
    match self.head.take() {
      Some(mut head) => {
        self.head = head.take_next();
        Some(head)
      },
      None => None,
    }
  }

  /// Remove all elements matching `predicate` and return them in a new list
  /// O(n) algorithmic time
  pub fn remove<F: Fn(&T) -> bool>(&mut self, predicate: F) -> SortedList<T> {
    let mut matching = SortedList::new();
    let mut not_matching = SortedList::new();

    while let Some(mut head) = self.head.take() {
      self.head = head.take_next();

      if predicate(&head) {
        matching.insert(head);
      }
      else {
        not_matching.insert(head);
      }
    }
    *self = not_matching;
    matching
  }
  
  /*
  /// Append all the elements of `queue` onto self.
  /// O(1) algorithmic time
  pub fn append(&mut self, mut queue: SortedQueue<T>) {
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
  */

  /*
  /// Modify all the elements of the queue with the block passed in.
  /// O(n) algorithmic time
  pub fn modify_all<F: Fn(&mut T)>(&mut self, block: F) {
    let mut current = self.head.as_mut();
    while let Some(node) = current {
      block(&mut *node);
      current = node.next_mut();
    }
  }
  */

  /// Remove all the elements from `self` and return it in a SortedList.
  /// O(1) algorithmic time
  pub fn remove_all(&mut self) -> SortedList<T> {
    ::core::mem::replace(self, SortedList::new())
  }

  /// Return true if the list is empty, false otherwise.
  /// O(1) algorithmic time
  pub fn is_empty(&self) -> bool {
    self.head.is_none()
  }
}
