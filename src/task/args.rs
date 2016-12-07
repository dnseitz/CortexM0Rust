// task/args.rs
// AltOSRust
//
// Created by Daniel Seitz on 12/7/16

use core::any::Any;
use core::ops::{Index, IndexMut};
use collections::Vec;
use alloc::boxed::Box;

pub struct Empty;

pub struct ArgsBuilder<T: Any> {
  cap: usize,
  len: usize,
  vec: Vec<Box<T>>,
}

impl ArgsBuilder<Empty> {
  pub fn empty() -> Args<Empty> {
    Args::empty()
  }
}

impl<T: Any> ArgsBuilder<T> {
  pub fn new(cap: usize) -> Self {
    ArgsBuilder { 
      cap: cap,
      len: 0,
      vec: Vec::with_capacity(cap),
    }
  }

  #[inline(never)]
  pub fn add_arg(mut self, arg: T) -> Self {
    if self.len >= self.cap {
      panic!("ArgsBuilder::add_arg - added too many arguments!");
    }
    unsafe { 
      let cell = self.vec.get_unchecked_mut(self.len);
      *cell = Box::new(arg);
    }
    self.len += 1;
    self
  }

  pub fn finalize(mut self) -> Args<T> {
    unsafe { self.vec.set_len(self.len) };
    Args::new(self.vec)  
  }
}

pub struct Args<T: Any> {
  args: Vec<Box<T>>,
}

impl Args<Empty> {
  pub fn empty() -> Self {
    Args { args: Vec::with_capacity(0) }
  }
}

impl<T: Any> Args<T> {
  fn new(args: Vec<Box<T>>) -> Self {
    Args { args: args }
  }

  pub fn as_ptr(&self) -> *const Self {
    self
  }

}

impl<T: Any> Index<usize> for Args<T> {
  type Output = T;
  fn index(&self, index: usize) -> &Self::Output {
    /*
    if self.args.len() <= index {
      panic!("Args::index - index {} out of bounds!", index);
    }
    unsafe { &*self.args.get_unchecked(index) }
    */
    &*self.args[index]
  }
}

impl<T: Any> IndexMut<usize> for Args<T> {
  fn index_mut(&mut self, index: usize) -> &mut Self::Output {
    /*
    if self.args.len() <= index {
      panic!("Args::index_mut - index {} out of bounds!", index);
    }
    unsafe { &mut *self.args.get_unchecked_mut(index) }
    */
    &mut *self.args[index]
  }
}
