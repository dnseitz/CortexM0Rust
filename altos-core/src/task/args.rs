// task/args.rs
// AltOSRust
//
// Created by Daniel Seitz on 12/7/16

//! Arguments used in tasks.
//!
//! This module contains implementations for structs that help pass arguments into a task. The
//! `Builder` struct provides an interface specifying what values the arguments to a task should
//! have. Begin by specifying how many arguments a task should take by creating a new `Builder`
//! with that capacity, and use the `add_arg()` method to give each argument a value. Once you have
//! added all the arguments required, call the `finalize()` method to finish up the creation and
//! return a usable `Args` object. For example:
//!
//! ```rust,no_run
//! use altos_core::task::args::{Builder, Args};
//! use altos_core::task::{Priority, new_task};
//!
//! let mut args = Builder::new(2);
//! args = args.add_arg(100).add_arg(500);
//!
//! new_task(test_task, args.finalize(), 512, Priority::Normal, "args");
//!
//! fn test_task(args: &Args) {
//!   let first = args[0]; // first = &100
//!   let second = args[1]; // secont = &500
//!   loop {}
//! }
//! ```

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
  /// Returns an empty `Args` object.
  ///
  /// Use this if the task you are creating should not take any arguments.
  pub fn empty() -> Args {
    Args::empty()
  }

  /// Creates a new builder with the specified capacity.
  ///
  /// The number of arguments for a task should be known before hand in order to avoid unnecessary
  /// reallocations. Attempting to exceed this capacity will panic the kernel.
  pub fn new(cap: usize) -> Self {
    Builder { 
      cap: cap,
      len: 0,
      vec: Vec::with_capacity(cap),
    }
  }

  /// Adds an argument to the list of arguments.
  ///
  /// The argument should be either an integer value or a pointer casted as an integer. When using
  /// the arguments within the task you must know the type and order of each argument and cast them
  /// manually to the correct object.
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// use altos_core::task::args::Builder;
  ///
  /// let mut args = Builder::new(2);
  /// args = args.add_arg(100).add_arg(500);
  /// ```
  ///
  /// # Panics
  ///
  /// This method will panic if you attempt to add more arguments than the capacity allocated.
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

  /// Returns a finalized `Args` object.
  ///
  /// After adding all the arguments that are required for the task, call this method to finalize
  /// the construction of the object. The finalized object will be used in the creation of a new
  /// task.
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// use altos_core::task::args::Builder;
  ///
  /// let mut args = Builder::new(2);
  /// args = args.add_arg(100).add_arg(500);
  /// let finalized_args = args.finalize();
  /// ```
  pub fn finalize(mut self) -> Args {
    unsafe { self.vec.set_len(self.len) };
    Args::new(self.vec)  
  }
}

/// An object representing all of the arguments passed into a task.
/// 
/// This object contains a list of raw integer values that can be either interpreted as integer
/// values or raw pointer values that can be later casted into references. The task must know the
/// order and type of arguments passed into it in order to correctly interpret them. Unfortunately
/// we can not keep type safety across the task initialization barrier in order to keep tasks
/// uniform.
pub struct Args {
  args: Vec<Box<RawPtr>>,
}

impl Args {
  /// Returns an empty `Args` object.
  ///
  /// Use this when a task doesn't require any arguments.
  pub fn empty() -> Self {
    Args { args: Vec::with_capacity(0) }
  }

  fn new(args: Vec<Box<RawPtr>>) -> Self {
    Args { args: args }
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
