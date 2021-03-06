// lib.rs
// AltOSRust
//
// Created by Daniel Seitz on 11/30/16

#![feature(const_fn)]
#![no_std]
#![allow(dead_code)]

extern crate cortex_m0;

use cortex_m0::peripheral::gpio;
use cortex_m0::kernel::sync::{Mutex, MutexGuard};
use cortex_m0::kernel::sync::CondVar;

use cortex_m0::time;
use cortex_m0::kernel::task::args::{Args, ArgsBuilder};
use cortex_m0::kernel::task;
use cortex_m0::kernel::syscall;
use cortex_m0::kernel::task::TaskHandle;
use cortex_m0::kernel::alloc::Box;
use cortex_m0::arm;

#[no_mangle]
// FIXME: Unmangle and make private again
pub static TEST_MUTEX: Mutex<u32> = Mutex::new(0);
pub static TEST_CONDVAR: CondVar = CondVar::new();

#[no_mangle]
pub fn application_entry() -> ! {
  let mut args = ArgsBuilder::with_capacity(2);
  //     rate      frequency
  args.add_num(50).add_num(5);

  let guard = TEST_MUTEX.lock();
  let mut condvar_args = ArgsBuilder::with_capacity(1);
  condvar_args.add_box(Box::new(guard));

  syscall::new_task(condvar_waiter, condvar_args.finalize(), 512, task::Priority::Critical, "condvar wait task");
  syscall::new_task(condvar_notifier, Args::empty(), 512, task::Priority::Critical, "condvar notify task");
  //syscall::new_task(test_task_1, Args::empty(), 512, task::Priority::Critical, "first task");
  //syscall::new_task(test_task_2, Args::empty(), 512, task::Priority::Critical, "second task");
  //syscall::new_task(test_task_3, Args::empty(), 512, task::Priority::Critical, "third task");
  //syscall::new_task(mutex_task_1, Args::empty(), 1024, task::Priority::Critical, "first mutex task");
  //syscall::new_task(mutex_task_2, Args::empty(), 1024, task::Priority::Critical, "second mutex task");
  //syscall::new_task(delay_task, 512, task::Priority::Critical, "delay task");
  //syscall::new_task(frequency_task_1, 512, task::Priority::Critical, "frequency task 1");
  //syscall::new_task(frequency_task_2, 512, task::Priority::Critical, "frequency task 2");
  //syscall::new_task(preempt_task_1, 512, task::Priority::Critical, "preempt task 1");
  //syscall::new_task(preempt_task_2, 512, task::Priority::Critical, "preempt task 2");
  //syscall::new_task(arg_task, args.finalize(), 512, task::Priority::Critical, "arg task");
  //let handle = syscall::new_task(to_destroy, Args::empty(), 512, task::Priority::Critical, "to destroy");
  //let mut destroy_args = ArgsBuilder::new(1);
  //destroy_args = destroy_args.add_box(Box::new(handle));
  //syscall::new_task(destroy_task, destroy_args.finalize(), 512, task::Priority::Critical, "destroy task");
  task::start_scheduler();

  loop { unsafe { arm::asm::bkpt() }; }
}

fn condvar_waiter(args: &mut Args) {
  let pb3 = gpio::Port::new(3, gpio::Group::B);
  let mut guard = unsafe { *args.pop_box::<MutexGuard<'static, u32>>() };
  loop {
    guard = TEST_CONDVAR.wait(guard);
    pb3.set(); 
    time::delay_ms(*guard as usize);
    pb3.reset();
    time::delay_ms(*guard as usize);
  }
}

fn condvar_notifier(_args: &mut Args) {
  loop {
    let mut guard = TEST_MUTEX.lock();
    if *guard >= 2000 {
      *guard = 100;
    }
    else {
      *guard += 100;
    }
    TEST_CONDVAR.notify_all();
    drop(guard);
    time::delay_ms(4000);
  }
}

fn delay_task() {
  let pb3 = gpio::Port::new(3, gpio::Group::B);
  loop {
    pb3.set();
    time::delay_ms(100);
    pb3.reset();
    time::delay_ms(100);
  }
}

fn mutex_task_1(_args: &mut Args) {
  let pb3 = gpio::Port::new(3, gpio::Group::B);
  let mut value: u32 = 0;
  loop {
    value = value.wrapping_add(0x1);
    value = value & 0xFFFF;
    let mut guard = TEST_MUTEX.lock();
    if value == 0xFFFF {
      pb3.set();
      time::delay_ms(2000);
      pb3.reset();
    }
    *guard = *guard & 0xFFFF0000;
    *guard = *guard | value;
    drop(guard);
  }
}

fn mutex_task_2(_args: &mut Args) {
  let pb3 = gpio::Port::new(3, gpio::Group::B);
  let mut value: u32 = 0;
  loop {
    value = value.wrapping_add(0x10000);
    value = value & 0xFFFF0000;
    let mut guard = TEST_MUTEX.lock();
    if value == 0xFFFF0000 {
      for _ in 0..10 {
        pb3.set();
        time::delay_ms(100);
        pb3.reset();
        time::delay_ms(100);
      }
    }
    *guard = *guard & 0xFFFF;
    *guard = *guard | value;
    drop(guard);
  }
}

fn test_task_1(_args: &mut Args) {
  let pb3 = gpio::Port::new(3, gpio::Group::B);
  loop {
    for _ in 0..5 {
      pb3.set();
      time::delay_ms(100);
      pb3.reset();
      time::delay_ms(100);
    }
  }
}

fn test_task_2(_args: &mut Args) {
  let pb3 = gpio::Port::new(3, gpio::Group::B);
  loop {
    for _ in 0..3 {
      pb3.set();
      time::delay_ms(500);
      pb3.reset();
      time::delay_ms(500);
    }
  }
}

fn test_task_3(_args: &mut Args) {
  let pb3 = gpio::Port::new(3, gpio::Group::B);
  loop {
    for _ in 0..10 {
      pb3.set();
      time::delay_ms(50);
      pb3.reset();
      time::delay_ms(50);
    }
  }
}

fn frequency_task_1() {
  let pb3 = gpio::Port::new(3, gpio::Group::B);
  let delay = 500;
  loop {
    pb3.set();
    time::delay_ms(delay);
    pb3.reset();
    time::delay_ms(delay);
  }
}

fn frequency_task_2() {
  let pb3 = gpio::Port::new(3, gpio::Group::B);
  let mut delay = 250;
  loop {
    pb3.set();
    time::delay_ms(delay);
    pb3.reset();
    time::delay_ms(delay);
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

fn arg_task(args: &mut Args) {
  let rate = args.pop_num();
  let multiplier = args.pop_num();
  let pb3 = gpio::Port::new(3, gpio::Group::B);
  loop {
    pb3.set();
    time::delay_ms(rate * multiplier);
    pb3.reset();
    time::delay_ms(rate * multiplier);
  }
}

fn destroy_task(args: &mut Args) {
  let mut handle = unsafe { args.pop_box::<TaskHandle>() };
  let pb3 = gpio::Port::new(3, gpio::Group::B);
  loop {
    pb3.set();
    time::delay_ms(1000);
    pb3.reset();
    time::delay_ms(1000);
    handle.destroy();
  }
}

fn to_destroy(_args: &mut Args) {
  let pb3 = gpio::Port::new(3, gpio::Group::B);
  loop {
    pb3.set();
    time::delay_ms(100);
    pb3.reset();
    time::delay_ms(100);
  }
}
