#![feature(lang_items)]
#![feature(core_intrinsics)]
#![feature(asm)]
#![feature(naked_functions)]
#![feature(const_fn)]
#![feature(alloc)]
#![feature(collections)]
#![feature(drop_types_in_const)] // Probably can come back and remove this later
#![no_std]

extern crate bump_allocator;
extern crate alloc;
#[macro_use]
extern crate collections;

mod exceptions;
mod peripheral;
mod math;
mod timer;
mod volatile;
mod arm;
mod interrupt;
mod task;
mod system_control;

use peripheral::gpio;
use peripheral::rcc;
use peripheral::systick;

pub use math::{__aeabi_uidiv, __aeabi_uidivmod};
#[cfg(not(test))]
pub use vector_table::RESET;
#[cfg(not(test))]
pub use exceptions::EXCEPTIONS;
pub use task::{current_task, switch_context};

#[no_mangle]
pub fn start() -> ! {
  init_data_segment();
  init_bss_segment();
  bump_allocator::init_heap();
  gpio::GPIO::enable(gpio::Group::B);

  let a_box = alloc::boxed::Box::new(42);

  let mut pb3 = gpio::Port::new(3, gpio::Group::B);
  pb3.set_mode(gpio::Mode::Output);
  pb3.set_type(gpio::Type::PushPull);

  let rcc = rcc::rcc();
  let systick = systick::systick();
  
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
  
  systick.use_processor_clock();
  systick.clear_current_value();
  systick.enable_counter();
  systick.enable_interrupts();

  task::init();
  task::new_task(test_task_1, 512, task::Priority::Critical, "first task");
  task::new_task(test_task_2, 512, task::Priority::Critical, "second task");
  task::new_task(test_task_3, 512, task::Priority::Critical, "third task");
  task::start_first_task();

  loop { unsafe { arm::bkpt() }; }
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
    task::yield_task();
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
    task::yield_task();
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
    task::yield_task();
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
    : "r0", "r1", "r2", "r3" /* clobbers */);  
  }
}

fn init_bss_segment() {
  unsafe {
    asm!(
      concat!(
      "movs r0, #0\n",
      "ldr r1, =_sbss\n",
      "ldr r2, =_ebss\n",
    "loop:\n",
      "cmp r1, r2\n",
      "bpl b_done\n",
      "str r0, [r1]\n",
      "adds r1, #4\n",
      "b loop\n",
    "b_done:\n")
    : /* no outputs */
    : /* no inputs */
    : "r0", "r1", "r2" /* clobbers */);
  }
}
