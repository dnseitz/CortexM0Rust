// task/task_control.rs
// AltOSRust
//
// Created by Daniel Seitz

use alloc::boxed::Box;
use collections::Vec;
use volatile::Volatile;
use super::args::Args;
use super::NUM_PRIORITIES;


const VALID_TASK: usize = 0xBADB0100;
const INVALID_TASK: usize = 0x0;

type HandleResult<T> = Result<T, ()>;

#[derive(Copy, Clone)]
pub enum Priority {
  Critical = 0,
  Normal = 1,
  Low = 2,

  #[doc(hidden)]
  __Idle = 3,
}

impl Priority {
  pub fn all() -> ::core::ops::Range<usize> {
    (0..NUM_PRIORITIES)
  }

  pub fn higher(&self) -> ::core::ops::Range<usize> {
    0..(*self as usize + 1)
  }
}

#[derive(Copy, Clone, PartialEq)]
pub enum State {
  Embryo,
  Ready,
  Running,
  Blocked,
  Suspended,
}

#[repr(C)]
#[doc(hidden)]
pub struct TaskControl {
  stack: usize, /* stack pointer MUST be first field */
  stack_base: usize,
  stack_depth: usize,
  args: Option<Box<Args>>,
  tid: usize,
  name: &'static str,
  pub valid: usize,
  pub wchan: usize,
  pub delay: usize,
  pub destroy: bool,
  pub overflowed: bool,
  pub priority: Priority,
  pub state: State,
}

impl TaskControl {
  pub fn new(code: fn(&Args), args: Args, depth: usize, priority: Priority, name: &'static str) -> Self {
    let stack_mem: Vec<u8> = Vec::with_capacity(depth);
    // Arguments struct stored right above the stack
    let args_mem: Box<Args> = Box::new(args);

    let stack = stack_mem.as_ptr() as usize;
    // Don't free the heap space, we'll clean up when we drop the TaskControl
    ::core::mem::forget(stack_mem);
    let tid = tid::fetch_next_tid();
    let mut task = TaskControl {
      stack: stack + depth,
      stack_base: stack,
      stack_depth: depth,
      args: Some(args_mem),
      tid: tid,
      name: name,
      valid: VALID_TASK + (tid & 0xFF),
      wchan: 0,
      delay: 0,
      destroy: false,
      overflowed: false,
      priority: priority,
      state: State::Embryo,
    };
    task.initialize(code);
    task
  }

  const fn uninitialized(name: &'static str) -> Self {
    TaskControl {
      stack: 0,
      stack_base: 0,
      stack_depth: 0,
      args: None,
      tid: !0,
      name: name,
      valid: INVALID_TASK,
      wchan: 0,
      delay: 0,
      destroy: false, 
      overflowed: false,
      priority: Priority::Low,
      state: State::Embryo,
    }
  }

  /// This initializes the task's stack. This method MUST only be called once, calling it more than
  /// once could at best waste some stack space and at worst corrupt an active stack.
  fn initialize(&mut self, code: fn(&Args)) {
    unsafe {
      let stack_ptr = Volatile::new(self.stack as *const usize);
      self.stack = ::initialize_stack(stack_ptr, code, self.args.as_ref());
    }
    self.state = State::Ready;
  }

  /// Check if the stack has gone past its bounds
  pub fn is_stack_overflowed(&self) -> bool {
    // TODO: Add some stack guard bytes to check if we've overflowed during execution?
    //  This would add some extra overhead, maybe have some #[cfg] that determines if we should add
    //  this extra security?
    self.stack <= self.stack_base
  }
}

impl Drop for TaskControl {
  fn drop(&mut self) {
    // Rebuild stack vec then drop stack memory
    let size = self.stack_depth;
    unsafe { 
      drop(Vec::from_raw_parts(self.stack_base as *mut u8, size, size));
    }
  }
}

pub struct TaskHandle(*const TaskControl);

impl TaskHandle {
  pub fn new(task: *const TaskControl) -> Self {
    TaskHandle(task)
  }

  pub fn destroy(&self) -> bool {
    if self.is_valid() {
      let task = self.task_ref_mut();
      task.destroy = true;
      task.valid = INVALID_TASK;
      true
    }
    else {
      false
    }
  }

  pub fn priority(&self) -> HandleResult<Priority> {
    if self.is_valid() {
      let task = self.task_ref();
      Ok(task.priority)
    }
    else {
      Err(())
    }
  }

  pub fn state(&self) -> HandleResult<State> {
    if self.is_valid() {
      let task = self.task_ref();
      Ok(task.state)
    }
    else {
      Err(())
    }
  }

  pub fn tid(&self) -> HandleResult<usize> {
    if self.is_valid() {
      let task = self.task_ref();
      Ok(task.tid)
    }
    else {
      Err(())
    }
  }

  pub fn name(&self) -> HandleResult<&'static str> {
    if self.is_valid() {
      let task = self.task_ref();
      Ok(task.name)
    }
    else {
      Err(())
    }
  }

  pub fn stack_size(&self) -> HandleResult<usize> {
    if self.is_valid() {
      let task = self.task_ref();
      Ok(task.stack_depth)
    }
    else {
      Err(())
    }
  }

  fn is_valid(&self) -> bool {
    let (tid, valid) = unsafe { ((*self.0).tid, (*self.0).valid) };
    let tid_mask = tid & 0xFF;
    valid + tid_mask == VALID_TASK + tid_mask 
  }

  fn task_ref(&self) -> &TaskControl {
    unsafe { &*self.0 }
  }

  fn task_ref_mut(&self) -> &mut TaskControl {
    unsafe { &mut *(self.0 as *mut TaskControl) }
  }
}

mod tid {
  use atomic::{ATOMIC_USIZE_INIT, AtomicUsize, Ordering};

  static CURRENT_TID: AtomicUsize = ATOMIC_USIZE_INIT;
  
  /// Atomically increment the task id and return the old value
  pub fn fetch_next_tid() -> usize {
    CURRENT_TID.fetch_add(1, Ordering::SeqCst)
  }
}
