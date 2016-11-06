#![feature(lang_items)]
#![feature(asm)]
#![no_std]
#![no_main]

mod exceptions;
mod peripheral;

use peripheral::gpio;
use peripheral::rcc;

#[no_mangle]
pub fn start() -> ! {
  gpio::GPIO::enable(gpio::GPIOGroup::B);

  let mut pb3 = gpio::GPIOPort::new(3, gpio::GPIOGroup::B);
  pb3.set_mode(gpio::GPIOMode::Output);
  pb3.set_type(gpio::GPIOType::PushPull);

  // Just looking...
  let pb3_mode = pb3.get_mode();
  let pb3_type = pb3.get_type();

  let rcc = rcc::rcc();

  // Check system clock source...
  let clock_source: rcc::Clock = rcc.get_system_clock_source();

  let mut clock_multiplier: u8 = 12;

  rcc.disable_clock(rcc::Clock::PLL);
  rcc.set_pll_source(rcc::Clock::HSI);
  rcc.set_pll_multiplier(clock_multiplier);
  rcc.enable_clock(rcc::Clock::PLL);

  let pll_enabled = rcc.clock_is_on(rcc::Clock::PLL);

  while !rcc.clock_is_ready(rcc::Clock::PLL) {}
  rcc.set_system_clock_source(rcc::Clock::PLL);

  let new_clock_source: rcc::Clock = rcc.get_system_clock_source();

  let did_disable_hsi = rcc.disable_clock(rcc::Clock::HSI);
  
  let mut ticks: u32 = 5_000;
  loop {
    pb3.set();
    delay(ticks);
    pb3.reset();
    delay(ticks);
  }

}

fn delay(n: u32) {
  for _ in 0..n {}
}

mod vector_table {
  #[link_section = ".reset"]
  static RESET: fn() -> ! = ::start;
}

#[lang = "eh_personality"] extern fn eh_personality() {}
#[lang = "panic_fmt"] extern fn panic_fmt() -> ! {loop{}}
