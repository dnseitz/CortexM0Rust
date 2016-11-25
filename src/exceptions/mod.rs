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
      "mrs r0, psp
       
       ldr r3, current_task_const
       ldr r2, [r3]

       subs r0, r0, #32
       str r0, [r2]
       stmia r0!, {r4-r7}
        mov r4, r8
        mov r5, r9
        mov r6, r10
        mov r7, r11
        stmia r0!, {r4-r7}
        
       push {r3, r14}
       cpsid i
       bl switch_context
       cpsie i
       pop {r2, r3}

       ldr r1, [r2]
       ldr r0, [r1]
       adds r0, r0, #16
       ldmia r0!, {r4-r7}
        mov r8, r4
        mov r9, r5
        mov r10, r6
        mov r11, r7

       msr psp, r0

       subs r0, r0, #32
        ldmia r0!, {r4-r7}

       bx r3

        .align 4
       current_task_const: .word current_task
      "
    );
  }
}
