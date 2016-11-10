
use super::timer;
use arm::bkpt;

#[cfg(not(test))]
#[link_section = ".exceptions"]
#[no_mangle]
pub static EXCEPTIONS: [Option<fn()>; 14] = [Some(default_handler),  // NMI
                                              Some(default_handler),  // Hard Fault
                                              Some(default_handler),  // Memory Management Fault
                                              Some(default_handler),  // Bus Fault
                                              Some(default_handler),  // Usage Fault
                                              None,                   // Reserved
                                              None,                   // Reserved
                                              None,                   // Reserved
                                              None,                   // Reserved
                                              Some(default_handler),  // SVCall
                                              None,                   // Reserved for Debug
                                              None,                   // Reserved
                                              Some(default_handler),  // PendSV
                                              Some(systick_handler)]; // SysTick
                                              


pub fn default_handler() {
    unsafe { bkpt(); }
}

pub fn systick_handler() {
  timer::Timer::tick();
}
