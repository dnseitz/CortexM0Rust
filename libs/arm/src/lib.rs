// lib.rs
// AltOSRust
//
// Created by Daniel Seitz on 12/6/16

#![no_std]
#![feature(asm)]

mod math;
mod mem;
pub mod asm;

pub use math::*;
pub use mem::*;

/**************************************************************************************************
 * An Example implementation of the begin_critical and end_critical functions needed by AltOSRust
 * for handling of critical sections.
 *  ```
 *    #[no_mangle]
 *    pub extern fn begin_critical() -> usize {
 *      if cfg!(target_arch = "arm") {
 *        let primask: usize;
 *        unsafe {
 *          asm!(
 *            concat!(
 *              "mrs $0, PRIMASK\n",
 *              "cpsid i\n")
 *            : "=r"(primask)
 *            : /* no inputs */
 *            : /* no clobbers */
 *            : "volatile");
 *        }
 *        primask
 *      } else {
 *        0
 *      }
 *    }
 *
 *    #[no_mangle]
 *    pub extern fn end_critical(primask: usize) {
 *      #[cfg(target_arch="arm")]
 *      unsafe {
 *        asm!("msr PRIMASK, $0"
 *          : /* no outputs */
 *          : "r"(primask)
 *          : /* no clobbers */
 *          : "volatile");
 *      }
 *    }
 *  ```
 *************************************************************************************************/
