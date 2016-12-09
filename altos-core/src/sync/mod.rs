// sync/mod.rs
// AltOSRust
//
// Created by Daniel Seitz on 12/1/16

pub mod mutex;
pub mod spin;
mod critical;

pub use self::mutex::Mutex;
pub use self::spin::SpinMutex;
pub use self::critical::CriticalSection;
