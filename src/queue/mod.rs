// queue/mod.rs
// AltOSRust
//
// Created by Daniel Seitz on 12/2/16

mod queue;
mod atomic_queue;
mod sorted_list;

pub use self::queue::*;
pub use self::atomic_queue::*;
use alloc::boxed::Box;
use core::ops::{Deref, DerefMut};

#[repr(C)]
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

impl<T> Deref for Node<T> {
  type Target = T;
  fn deref(&self) -> &Self::Target {
    &self.data
  }
}

impl<T> DerefMut for Node<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.data
  }
}
