// task/public.rs
// AltOSRust
//
// Created by Daniel Seitz on 12/9/16

//! The public interface for the task module of the kernel.
//!
//! This module contains all the functions and types that should be public to any application using
//! the task module of AltOSRust.

pub use super::{new_task, start_scheduler};
pub use super::{TaskHandle, Priority};

#[allow(missing_docs)]
pub mod args {
  pub use super::super::{ArgsBuilder, Args};
}
