// peripheral/serial/mod.rs
// AltOSRust
//
// Created by Daniel Seitz on 11/30/16
mod usart_control;

use super::Control;
use volatile::Volatile;
use peripheral::{gpio, rcc};

fn init() {
  let rcc = rcc::rcc();
  let cr = rcc.get_system_clock_rate();

  gpio::GPIO::enable(gpio::GPIOGroup::A);
  rcc.enable_peripheral(rcc::Peripheral::USART1);

  let pa9 = gpio::GPIOPort::new(9, gpio::GPIOGroup::A);
  let pa10 = gpio::GPIOPort::new(10, gpio::GPIOGroup::A);

  pa9.set_function(gpio::AlternateFunction::One);
  pa10.set_function(gpio::AlternateFunction::One);

  pa9.set_speed(gpio::Speed::High);
  pa10.set_speed(gpio::Speed::High);

  pa9.set_mode(gpio::Mode::Alternate);
  pa10.set_mode(gpio::Mode::Alternate);

  pa9.set_type(gpio::Type::PushPull);
  pa10.set_type(gpio::Type::PushPull);

  pa9.set_pull(gpio::Pull::Up);
  pa10.set_pull(gpio::Pull::Up);


}

#[derive(Copy, Clone)]
enum USARTx {
    One,
    Two,
}

struct USART {
    mem_addr: usize,
    control: USARTControl,
}

impl Control for USART {
    unsafe fn mem_addr(&self) -> Volatile<usize> {
        Volatile::new(self.mem_addr as *usize)
    }
}

impl USART {
    fn usart(x: USARTx) -> Self {
        const USART1: usize = 0x4001_3800;
        const USART2: usize = 0x4000_4400;

        match x {
            USARTx::One => USART {
                    mem_addr: USART1,
                    control: USARTControl::new(USART1),
                },
            USARTx::Two => USART2 {
                    mem_addr = USART2,
                    control: USARTControl::new(USART2),
                },
        }
    }
}
