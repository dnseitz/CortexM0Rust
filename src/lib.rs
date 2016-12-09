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
#![feature(cfg_target_has_atomic)]
#![no_std]
#![allow(dead_code)]

#[cfg(not(test))]
extern crate bump_allocator;
extern crate alloc;
#[macro_use]
extern crate collections;
extern crate arm;
#[cfg(not(target_has_atomic="ptr"))]
extern crate cm0_atomic as atomic;

mod exceptions;
mod peripheral;
mod timer;
mod volatile;
mod interrupt;
mod task;
mod system_control;
mod sync;
mod queue;

#[cfg(target_has_atomic="ptr")]
use core::sync::atomic as atomic;
use peripheral::gpio;
use peripheral::rcc;
use peripheral::systick;
use sync::Mutex;
use task::TaskHandle;
use task::args::{Args, Builder};

#[cfg(not(test))]
pub use vector_table::RESET;
#[cfg(not(test))]
pub use exceptions::EXCEPTIONS;
pub use task::{CURRENT_TASK, switch_context};
use alloc::boxed::Box;

#[no_mangle]
// FIXME: Unmangle and make private again
pub static TEST_MUTEX: Mutex<u32> = Mutex::new(0);

// List of methods we'll likely need from port layer
extern {
  fn yield_cpu();
  fn initialize_stack(stack_ptr: volatile::Volatile<usize>, code: fn(&Args), args: Option<&Box<Args>>) -> usize;
  fn start_first_task();
  fn in_kernel_mode() -> bool;
}

#[no_mangle]
pub fn start() -> ! {
  // TODO: set pendsv and systick interrupts to lowest priority
  init_data_segment();
  init_bss_segment();
  init_heap();
  init_led();
  init_clock();
  init_ticks();

  /*
  let mut args1 = ArgsBuilder::new(1);
  args1 = args1.add_arg(0xAABBCCDDusize);
  */

  let mut args = Builder::new(1);

  //task::new_task(test_task_1, 512, task::Priority::Critical, "first task");
  //task::new_task(test_task_2, 512, task::Priority::Critical, "second task");
  //task::new_task(test_task_3, 512, task::Priority::Critical, "third task");
  //task::new_task(mutex_task_1, ArgsBuilder::empty(), 1024, task::Priority::Critical, "first mutex task");
  //task::new_task(mutex_task_2, ArgsBuilder::empty(), 1024, task::Priority::Critical, "second mutex task");
  //task::new_task(delay_task, 512, task::Priority::Critical, "delay task");
  //task::new_task(frequency_task_1, 512, task::Priority::Critical, "frequency task 1");
  //task::new_task(frequency_task_2, 512, task::Priority::Critical, "frequency task 2");
  //task::new_task(preempt_task_1, 512, task::Priority::Critical, "preempt task 1");
  //task::new_task(preempt_task_2, 512, task::Priority::Critical, "preempt task 2");
  //task::new_task(arg_task, args2.finalize(), 512, task::Priority::Critical, "arg task");
  let handle = task::new_task(to_destroy, Args::empty(), 512, task::Priority::Critical, "to destroy");
  args = args.add_arg(&handle as *const _ as usize);
  task::new_task(destroy_task, args.finalize(), 512, task::Priority::Critical, "destroy task");
  task::start_scheduler();

  loop { unsafe { arm::asm::bkpt() }; }
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

fn mutex_task_1(_args: &Args) {
  let pb3 = gpio::Port::new(3, gpio::Group::B);
  let mut value = 0;
  loop {
    value += 0x1;
    value = value & 0xFFFF;
    let mut guard = TEST_MUTEX.lock();
    if value == 0xFFFF {
      pb3.set();
      timer::Timer::delay_ms(2000);
      pb3.reset();
    }
    *guard = *guard & 0xFFFF0000;
    *guard = *guard | value;
    drop(guard);
  }
}

fn mutex_task_2(_args: &Args) {
  let pb3 = gpio::Port::new(3, gpio::Group::B);
  let mut value = 0;
  loop {
    value += 0x10000;
    value = value & 0xFFFF0000;
    let mut guard = TEST_MUTEX.lock();
    if value == 0xFFFF0000 {
      for _ in 0..10 {
        pb3.set();
        timer::Timer::delay_ms(100);
        pb3.reset();
        timer::Timer::delay_ms(100);
      }
    }
    *guard = *guard & 0xFFFF;
    *guard = *guard | value;
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

fn preempt_task_1() {
  let mut value: usize = 0;
  loop {
    value += 1;
    if value == value {} // Silence unused warning
  }
}

fn preempt_task_2() {
  let mut value: usize = !0;
  loop {
    value -= 1;
    if value == value {} // Silence unused warning
  }
}

fn arg_task(args: &Args) {
  let ref rate = args[0];
  let ref multiplier = args[1];
  let pb3 = gpio::Port::new(3, gpio::Group::B);
  loop {
    pb3.set();
    timer::Timer::delay_ms(*rate * *multiplier);
    pb3.reset();
    timer::Timer::delay_ms(*rate * *multiplier);
  }
}

fn destroy_task(args: &Args) {
  let handle = unsafe { &*(args[0] as *const TaskHandle) };
  let pb3 = gpio::Port::new(3, gpio::Group::B);
  loop {
    pb3.set();
    timer::Timer::delay_ms(1000);
    pb3.reset();
    timer::Timer::delay_ms(1000);
    handle.destroy();
  }
}

fn to_destroy(_args: &Args) {
  let pb3 = gpio::Port::new(3, gpio::Group::B);
  loop {
    pb3.set();
    timer::Timer::delay_ms(100);
    pb3.reset();
    timer::Timer::delay_ms(100);
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
#[lang = "panic_fmt"] extern fn panic_fmt() -> ! {loop{unsafe {arm::asm::bkpt();}}}

fn init_data_segment() {
  #[cfg(target_arch="arm")]
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
    : "r0", "r1", "r2", "r3" /* clobbers */
    : "volatile");  
  }
}

fn init_bss_segment() {
  #[cfg(target_arch="arm")]
  unsafe {
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

fn init_heap() {
  #[cfg(target_arch="arm")]
  unsafe {
    let heap_start: usize;
    let heap_size: usize;
    asm!(
      concat!(
        "ldr r0, =_heap_start\n",
        "ldr r1, =_heap_end\n",

        "subs r2, r1, r0\n")
        : "={r0}"(heap_start), "={r2}"(heap_size)
        : /* no inputs */
        : "r0", "r1", "r2"
    );
    bump_allocator::init_heap(heap_start, heap_size);
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
