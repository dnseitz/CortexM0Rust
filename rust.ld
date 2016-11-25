/*
GROUP(
  libc.a
  libnosys.a
  libgcc.a
)
*/

MEMORY
{
  RAM (xrw)     : ORIGIN = 0x20000000, LENGTH = 6K
  FLASH (rx)    : ORIGIN = 0x08000000, LENGTH = 32K
}

stack_size = 1024;
heap_size = 256;

_stack_start = ORIGIN(RAM) + LENGTH(RAM);
_stack_end = _stack_start - stack_size;

ENTRY(start);

SECTIONS
{
  .vector_table ORIGIN(FLASH) :
  {
    LONG(_stack_start)
    KEEP(*(.reset))
    KEEP(*(.exceptions))
  } > FLASH

  .text :
  {
    . = ALIGN(4);
    *(.text)
    *(.text*)
    *(.rodata)
    *(.rodata*)
    *(.glue_6)      /* Glue arm to thumb code */
    *(.glue_6t)     /* Glue thumb to arm code */
    . = ALIGN(4);
    _etext = .;
  } > FLASH

  .ARM.extab :
  {
    *(.ARM.extab*)
  } > FLASH

  /*
  .ARM.extab : { *(.ARM.extab* .gnu.linkonce.armextab.*) } > FLASH
  .ARM : {
    __exidx_start = .;
    *(.ARM.exidx*)
    __exidx_end = .;
  } > FLASH
  */

  /* Used by startup to initialize data (not yet though) */
  _sidata = .;

  /* Initialized data section */
  .data : AT (_sidata)
  {
    . = ALIGN(4);
    _sdata = .;     /* Create global symbol at start of data */
    *(.data)
    *(.data*)

    . = ALIGN(4);
    _edata = .;
  } > RAM

  /* Uninitialized data section */
  . = ALIGN(4);
  .bss :
  { 
    /* Used by startup to initialize .bss section (not yet though) */
    _sbss = .;
    __bss_start__ = _sbss;
    *(.bss)
    *(.bss*)
    *(COMMON)

    . = ALIGN(4);
    _ebss = .; /* Define a global symbol at bss end */
    __bss_end__ = _ebss;
  } > RAM

  . = ALIGN(4);
  .heap :
  {
    _heap_start = .;
    . = . + heap_size;
    _heap_end = .;
  } > RAM

  /DISCARD/ :
  {
    *(.ARM.exidx*)
    *(.note.gnu.build-id*)
  }
}

