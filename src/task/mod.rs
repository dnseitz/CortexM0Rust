
use ::volatile::Volatile;
use ::system_control;

// TODO: In the future, allocate new tasks on the heap, this is just for testing
static mut task_1: TaskControl = TaskControl::new(0x2000_1000, 512);
static mut task_2: TaskControl = TaskControl::new(0x2000_0800, 512); 

#[no_mangle]
pub static mut current_task: &'static TaskControl = unsafe { &task_1 };

/*
struct TaskHandle<'task> {
  task: &'task TaskControl,
}
*/

pub enum Priority {
  Critical,
}

pub fn init(first: fn(), second: fn()) {
  unsafe {
    task_1.initialize(first);
    task_2.initialize(second);
  }
}

#[repr(C)]
pub struct TaskControl {
  stack: *const u32, /* stack pointer MUST be first field */
  stack_base: *const u32,
  stack_depth: u32,
  priority: Priority,
  name: &'static str,
}

impl TaskControl {
  const fn new(stack: u32, depth: u32) -> Self {
    TaskControl {
      stack: stack as *const u32,
      stack_base: (stack - depth) as *const u32,
      stack_depth: depth,
      priority: Priority::Critical,
      name: "test_task",
    }
  }

  fn initialize(&mut self, code: fn()) {
    const INITIAL_XPSR: u32 = 0x0100_0000;
    unsafe {
      let mut stack_mut = Volatile::new(self.stack);
      // Offset added to account for way MCU uses stack on entry/exit of interrupts
      stack_mut -= 4;
      stack_mut.store(INITIAL_XPSR); /* xPSR */
      stack_mut -= 4;
      stack_mut.store(code as u32); /* PC */
      stack_mut -= 4;
      stack_mut.store(exit_error as u32); /* LR */
      stack_mut -= 20; /* R12, R3, R2, R1 */
      // *stack_mut = params; /* R0 */
      stack_mut -= 32; /* R11..R4 */
      self.stack = stack_mut.as_ptr();
    }
  }

  fn is_stack_overflowed(&self) -> bool {
    self.stack <= self.stack_base
  }
}

#[no_mangle]
pub fn switch_context() {
  unsafe {
    if current_task.is_stack_overflowed() {
      ::arm::bkpt();
    }
    if current_task as *const TaskControl == &task_1 as *const TaskControl {
      current_task = &task_2;
    }
    else {
      current_task = &task_1;
    }
  }
}

pub fn start_first_task() {
  unsafe {
    asm!(
      concat!(
          "ldr r2, current_task_const_2\n", /* get location of current_task */
          "ldr r3, [r2]\n",
          "ldr r0, [r3]\n",

          "adds r0, #32\n", /* discard everything up to r0 */
          "msr psp, r0\n", /* this is the new top of stack to use for the task */

          "movs r0, #3\n", /* switch to the psp stack */
          "msr CONTROL, r0\n", /* we're using psp instead of msp now */

          "isb\n", /* instruction barrier */

          "pop {r0-r5}\n", /* pop the registers that are saved automatically */
          "mov lr, r5\n", /* lr is now in r5, so put it back where it belongs */
          "pop {r3}\n", /* pop return address (old pc) into r3 */
          "pop {r2}\n", /* pop and discard xPSR */
          "cpsie i\n", /* first task has its context, so interrupts can be enabled */
          "bx r3\n", /* start executing user code */

           ".align 4\n",
          "current_task_const_2: .word current_task\n")
        );
  }
}

fn exit_error() -> ! {
  unsafe {
    ::arm::bkpt();
    loop{}
  }
}

pub fn yield_task() {
  let scb = system_control::scb();
  scb.set_pend_sv();
  scb.clear_pend_sv();
}
