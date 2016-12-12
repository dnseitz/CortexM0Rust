// sync/mod.rs
// AltOSRust
//
// Created by Daniel Seitz on 12/1/16

//! Synchronization primitives for the AltOSRust kernel.
//!
//! This module implements several synchronization primitives for the kernel as well as
//! applications that rely on the kernel. They are used to control access to shared resources
//! across threads in order to avoid any data races.

pub mod mutex;
pub mod spin;
mod critical;

pub use self::mutex::Mutex;
pub use self::spin::SpinMutex;
pub use self::critical::CriticalSection;
