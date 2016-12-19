// task/public.rs
// AltOSRust
//
// Created by Daniel Seitz on 12/9/16

//! The public interface for the task module of the kernel.
//!
//! This module contains all the functions and types that should be public to any application using
//! the task module of AltOSRust.

pub use super::{yield_task, new_task, start_scheduler, system_tick};
pub use super::{TaskHandle, Priority};

#[allow(missing_docs)]
pub mod args {
  pub use super::super::{Builder, Args};
}
