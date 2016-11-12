#![feature(lang_items)]
#![feature(core_intrinsics)]
#![feature(asm)]
#![no_std]

mod exceptions;
mod peripheral;
mod math;
mod timer;
mod volatile;
mod arm;

use peripheral::gpio;
use peripheral::rcc;
use peripheral::systick;

pub use math::{__aeabi_uidiv, __aeabi_uidivmod};
#[cfg(not(test))]
pub use vector_table::RESET;
#[cfg(not(test))]
pub use exceptions::EXCEPTIONS;

#[no_mangle]
pub fn start() -> ! {
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


#[cfg(test)]
#[no_mangle]
pub fn _main() {}
