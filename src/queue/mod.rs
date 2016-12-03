// queue/mod.rs
// AltOSRust
//
// Created by Daniel Seitz on 12/2/16

mod queue;
mod atomic_queue;

pub use self::queue::*;
pub use self::atomic_queue::*;
use alloc::boxed::Box;

pub trait Queueable {
  fn set_next(&mut self, Option<Box<Self>>);
  fn take_next(&mut self) -> Option<Box<Self>>;
}

