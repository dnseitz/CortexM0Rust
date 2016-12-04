// lib.rs
// AltOSRust
//
// Created by Daniel Seitz on 11/30/16

#![feature(lang_items)]
#![feature(core_intrinsics)]
#![feature(asm)]
#![feature(naked_functions)]
#![feature(const_fn)]
#![feature(alloc)]
#![feature(collections)]
#![feature(drop_types_in_const)] // Probably can come back and remove this later
#![no_std]
//#![allow(dead_code)]

#[cfg(not(test))]
extern crate bump_allocator;
extern crate alloc;
#[macro_use]
extern crate collections;

#[macro_use]
mod atomic;
mod exceptions;
mod peripheral;
mod math;
mod timer;
mod volatile;
mod arm;
mod interrupt;
mod task;
mod system_control;
mod sync;
mod queue;

use peripheral::gpio;
use peripheral::rcc;
use peripheral::systick;
use sync::Mutex;

pub use math::{__aeabi_uidiv, __aeabi_uidivmod, __aeabi_lmul, __aeabi_memclr4};
#[cfg(not(test))]
pub use vector_table::RESET;
#[cfg(not(test))]
pub use exceptions::EXCEPTIONS;
pub use task::{CURRENT_TASK, switch_context};

#[no_mangle]
// FIXME: Unmangle and make private again
pub static TEST_MUTEX: Mutex<u32> = Mutex::new(0);

#[no_mangle]
pub fn start() -> ! {
  // TODO: set pendsv and systick interrupts to lowest priority
  init_data_segment();
  init_bss_segment();

  #[cfg(not(test))]
  bump_allocator::init_heap();

  init_led();
  init_clock();
  init_ticks();


  //task::new_task(test_task_1, 512, task::Priority::Critical, "first task");
  //task::new_task(test_task_2, 512, task::Priority::Critical, "second task");
  //task::new_task(test_task_3, 512, task::Priority::Critical, "third task");
  task::new_task(mutex_task_1, 512, task::Priority::Critical, "first mutex task");
  task::new_task(mutex_task_2, 512, task::Priority::Critical, "second mutex task");
  //task::new_task(delay_task, 512, task::Priority::Critical, "delay task");
  //task::new_task(frequency_task_1, 512, task::Priority::Critical, "frequency task 1");
  //task::new_task(frequency_task_2, 512, task::Priority::Critical, "frequency task 2");
  task::start_first_task();

  loop { unsafe { arm::bkpt() }; }
}

fn delay_task() {
  let pb3 = gpio::Port::new(3, gpio::Group::B);
  loop {
    pb3.set();
    timer::Timer::delay_ms(100);
    pb3.reset();
    timer::Timer::delay_ms(100);
  }
}

fn mutex_task_1() {
  loop {
    task::yield_task();
    let mut guard = TEST_MUTEX.lock();
    *guard = *guard + 1;
    task::yield_task();
    drop(guard);
  }
}

fn mutex_task_2() {
  loop {
    task::yield_task();
    let mut guard = TEST_MUTEX.lock();
    *guard = *guard + 10;
    task::yield_task();
    drop(guard);
  }
}

fn test_task_1() {
  let pb3 = gpio::Port::new(3, gpio::Group::B);
  loop {
    for _ in 0..5 {
      pb3.set();
      timer::Timer::delay_ms(100);
      pb3.reset();
      timer::Timer::delay_ms(100);
    }
  }
}

fn test_task_2() {
  let pb3 = gpio::Port::new(3, gpio::Group::B);
  loop {
    for _ in 0..3 {
      pb3.set();
      timer::Timer::delay_ms(500);
      pb3.reset();
      timer::Timer::delay_ms(500);
    }
  }
}

fn test_task_3() {
  let pb3 = gpio::Port::new(3, gpio::Group::B);
  loop {
    for _ in 0..10 {
      pb3.set();
      timer::Timer::delay_ms(50);
      pb3.reset();
      timer::Timer::delay_ms(50);
    }
  }
}

fn frequency_task_1() {
  let pb3 = gpio::Port::new(3, gpio::Group::B);
  let delay = 500;
  loop {
    pb3.set();
    timer::Timer::delay_ms(delay);
    pb3.reset();
    timer::Timer::delay_ms(delay);
  }
}

fn frequency_task_2() {
  let pb3 = gpio::Port::new(3, gpio::Group::B);
  let mut delay = 250;
  loop {
    pb3.set();
    timer::Timer::delay_ms(delay);
    pb3.reset();
    timer::Timer::delay_ms(delay);
    delay += 10;
    if delay > 750 {
      delay = 250;
    }
  }
}

mod vector_table {
  #[cfg(not(test))]
  #[link_section = ".reset"]
  #[no_mangle]
  pub static RESET: fn() -> ! = ::start;
}

#[cfg(not(test))]
#[lang = "eh_personality"] extern fn eh_personality() {}
#[cfg(not(test))]
#[lang = "panic_fmt"] extern fn panic_fmt() -> ! {loop{}}

fn init_data_segment() {
  unsafe {
    #![cfg(target_arch="arm")]
    asm!(
      concat!(
        "ldr r1, =_sidata\n", /* start of data in flash */
        "ldr r2, =_sdata\n",  /* start of memory location in RAM */
        "ldr r3, =_edata\n",  /* end of memory location in RAM */
      "copy:\n",
        "cmp r2, r3\n", /* check if we've reached the end of our segment */
        "bpl d_done\n",
        "ldr r0, [r1]\n", /* if not, keep copying */
        "adds r1, #4\n",
        "str r0, [r2]\n",
        "adds r2, #4\n",
        "b copy\n", /* repeat until done */
      "d_done:\n")
    : /* no outputs */ 
    : /* no inputs */ 
    : "r0", "r1", "r2", "r3" /* clobbers */
    : "volatile");  
  }
}

fn init_bss_segment() {
  unsafe {
    #![cfg(target_arch="arm")]
    asm!(
      concat!(
        "movs r0, #0\n", /* store zero for later */
        "ldr r1, =_sbss\n", /* start of bss in RAM */
        "ldr r2, =_ebss\n", /* end of bss in RAM */
      "loop:\n",
        "cmp r1, r2\n", /* check if we've reached the end of our segment */
        "bpl b_done\n",
        "str r0, [r1]\n", /* if not, zero out memory at current location */
        "adds r1, #4\n",
        "b loop\n", /* repeat until done */
      "b_done:\n")
    : /* no outputs */
    : /* no inputs */
    : "r0", "r1", "r2" /* clobbers */
    : "volatile");
  }
}

fn init_led() {
  gpio::GPIO::enable(gpio::Group::B);

  let mut pb3 = gpio::Port::new(3, gpio::Group::B);
  pb3.set_mode(gpio::Mode::Output);
  pb3.set_type(gpio::Type::PushPull);
}

fn init_clock() {
  let rcc = rcc::rcc();

  // 12 is the max we can go since our input clock is (8MHz / 2)
  let clock_multiplier: u8 = 12;

  // PLL must be off in order to configure
  rcc.disable_clock(rcc::Clock::PLL);

  // Make sure HSI is the PLL source clock
  rcc.set_pll_source(rcc::Clock::HSI);

  // Set the multiplier... DO NOT EXCEED 48 MHz
  rcc.set_pll_multiplier(clock_multiplier);

  // Enable the PLL clock
  rcc.enable_clock(rcc::Clock::PLL);

  // Wait for it to be ready
  while !rcc.clock_is_ready(rcc::Clock::PLL) {}
  // Switch over to the PLL for running the system
  rcc.set_system_clock_source(rcc::Clock::PLL);
}

fn init_ticks() {
  let systick = systick::systick();

  systick.use_processor_clock();
  systick.clear_current_value();
  systick.enable_counter();
  systick.enable_interrupts();

}
