// task/args.rs
// AltOSRust
//
// Created by Daniel Seitz on 12/7/16

use core::ops::{Index, IndexMut};
use collections::Vec;
use alloc::boxed::Box;

type RawPtr = usize;

/// An Args Builder.
///
/// Use this to construct a new list of arguments to pass into a task. The arguments should be
/// either a pointer to an object or a word length integer.
pub struct Builder {
  cap: usize,
  len: usize,
  vec: Vec<Box<RawPtr>>,
}

impl Builder {
  pub fn empty() -> Args {
    Args::empty()
  }

  pub fn new(cap: usize) -> Self {
    Builder { 
      cap: cap,
      len: 0,
      vec: Vec::with_capacity(cap),
    }
  }

  //#[inline(never)]
  pub fn add_arg(mut self, arg: RawPtr) -> Self {
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

  pub fn finalize(mut self) -> Args {
    unsafe { self.vec.set_len(self.len) };
    Args::new(self.vec)  
  }
}

pub struct Args {
  args: Vec<Box<RawPtr>>,
}

impl Args {
  pub fn empty() -> Self {
    Args { args: Vec::with_capacity(0) }
  }

  fn new(args: Vec<Box<RawPtr>>) -> Self {
    Args { args: args }
  }

  pub fn as_ptr(&self) -> *const Self {
    self
  }

}

impl Index<usize> for Args {
  type Output = usize;
  fn index(&self, index: usize) -> &Self::Output {
    &*self.args[index]
  }
}

impl IndexMut<usize> for Args {
  fn index_mut(&mut self, index: usize) -> &mut Self::Output {
    &mut *self.args[index]
  }
}
