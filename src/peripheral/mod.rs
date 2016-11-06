//! This module handles the memory mapped peripherals that are a part of the Cortex-M0. Submodules
//! will handle the more specific details of each peripheral.

pub mod rcc;
pub mod gpio;

pub trait Peripheral {
  fn mem_addr(&self) -> u32;
}

pub trait Register {
  fn new(base_addr: u32) -> Self;

  fn mem_offset(&self) -> u32;
}
