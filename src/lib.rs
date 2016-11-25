#![feature(lang_items)]
#![feature(core_intrinsics)]
#![feature(asm)]
#![feature(naked_functions)]
#![feature(const_fn)]
#![no_std]

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
  gpio::GPIO::enable(gpio::Group::B);

  let mut pb3 = gpio::Port::new(3, gpio::Group::B);
  pb3.set_mode(gpio::Mode::Output);
  pb3.set_type(gpio::Type::PushPull);

  let rcc = rcc::rcc();
  let systick = systick::systick();
  
  // 12 is the max we can go since our input clock is (8MHz / 2)
  let mut clock_multiplier: u8 = 12;

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

  let clock_rate = rcc.get_system_clock_rate();
  
  systick.use_processor_clock();
  systick.clear_current_value();
  systick.enable_counter();
  systick.enable_interrupts();

  task::init(test_task_1, test_task_2);

  task::start_first_task();
  
  //task::yield_task();

  loop { unsafe { arm::bkpt() }; }

  /*
  let mut ms_delay: u32 = 500;
  loop {
    //let timer = timer::Timer::get_current();
    pb3.set();
    timer::Timer::delay_ms(ms_delay);
    pb3.reset();
    timer::Timer::delay_ms(ms_delay);
    ms_delay += 50;
    if ms_delay > 1000 {
      ms_delay = 0;
    }
  }
  */
}

fn test_task_1() {
  let mut pb3 = gpio::Port::new(3, gpio::Group::B);
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
  let mut pb3 = gpio::Port::new(3, gpio::Group::B);
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
    asm!("
      ldr r1, =_sidata
      ldr r2, =_sdata
      ldr r3, =_edata
    copy:
      cmp r2, r3
      bpl done
      ldr r0, [r1]
      adds r1, #4
      str r0, [r2]
      adds r2, #4
      b copy
    done:
    "
    : /* no outputs */ 
    : /* no inputs */ 
    : "r0", "r1", "r2", "r3");  
  }
}
