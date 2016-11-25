#![cfg(not(test))]

use super::timer;
use arm::bkpt;

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
                                              Some(pend_sv_handler),  // PendSV
                                              Some(systick_handler)]; // SysTick
                                              


pub fn default_handler() {
    unsafe { bkpt(); }
}

pub fn systick_handler() {
  timer::Timer::tick();
}

/// Tell OS to context switch tasks
#[naked]
pub fn pend_sv_handler() {
  unsafe {
    asm!(
      concat!(
       "mrs r0, psp\n", /* move program stack pointer into r0 */
       
       "ldr r3, current_task_const\n", /* get the location of the current task struct */
       "ldr r2, [r3]\n",

       "subs r0, r0, #32\n", /* make space for the remaining low registers (r0-r3 saved
                                automatically) */
       "str r0, [r2]\n",     /* save new top of stack */
       "stmia r0!, {r4-r7}\n", /* store the low registers */
        "mov r4, r8\n", /* store the high registers */
        "mov r5, r9\n",
        "mov r6, r10\n",
        "mov r7, r11\n",
        "stmia r0!, {r4-r7}\n",
        
       "push {r3, r14}\n", /* store pointer to current task and lr on main stack */
       "cpsid i\n", /* disable interrupts for context switch */
       "bl switch_context\n",
       "cpsie i\n", /* re-enable interrupts */
       "pop {r2, r3}\n", /* pointer to current task now in r2, lr goes in r3 */

       "ldr r1, [r2]\n",
       "ldr r0, [r1]\n", /* get the task's top of stack in r0 */
       "adds r0, r0, #16\n", /* move to the high registers first */
       "ldmia r0!, {r4-r7}\n", /* pop the high registers */
        "mov r8, r4\n",
        "mov r9, r5\n",
        "mov r10, r6\n",
        "mov r11, r7\n",

       "msr psp, r0\n", /* store the new top of stack into program stack pointer */

       "subs r0, r0, #32\n", /* go back for the low registers not automatically stored */
        "ldmia r0!, {r4-r7}\n",

       "bx r3\n", /* return from context switch */

        ".align 4\n",
       "current_task_const: .word current_task\n")
    );
  }
}
