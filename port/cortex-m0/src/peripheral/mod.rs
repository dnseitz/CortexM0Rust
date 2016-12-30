// peripheral/mod.rs
// AltOSRust
//
// Created by Daniel Seitz on 11/30/16

//! This module handles the memory mapped peripherals that are a part of the Cortex-M0. Submodules
//! will handle the more specific details of each peripheral.

pub mod rcc;
pub mod gpio;
pub mod systick;
pub mod usart;

use volatile::Volatile;

pub trait Control {
  unsafe fn mem_addr(&self) -> Volatile<usize>;
}

pub trait Register {
  fn new(base_addr: usize) -> Self;

  fn base_addr(&self) -> usize;
  fn mem_offset(&self) -> usize;
  unsafe fn addr(&self) -> Volatile<usize> {
    Volatile::new((self.base_addr() + self.mem_offset()) as *const usize)
  }
}

pub trait Field {
  fn mask(&self) -> usize;
}
