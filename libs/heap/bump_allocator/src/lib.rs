
#![feature(allocator)]
#![feature(const_fn)]
#![feature(asm)]

#![allocator]
#![no_std]

static mut BUMP_ALLOCATOR: BumpAllocator = BumpAllocator::new();

/// Call this before doing any heap allocation
pub fn init_heap() {
  #[cfg(target_arch="arm")]
  unsafe {
    let heap_start: usize;
    let heap_size: usize;
    asm!(
      concat!(
        "ldr r0, =_heap_start\n",
        "ldr r1, =_heap_end\n",

        "subs r2, r1, r0\n")
        : "={r0}"(heap_start), "={r2}"(heap_size)
        : /* no inputs */
        : "r0", "r1", "r2"
    );
    BUMP_ALLOCATOR.init(heap_start, heap_size);
  }
}

struct BumpAllocator {
  heap_start: usize,
  heap_size: usize,
  next: usize,
}

impl BumpAllocator {
  /// Create a new bump allocator, which uses the memory in the range 
  /// [heap_start..heap_start + heap_size).
  const fn new() -> Self {
    BumpAllocator {
      heap_start: 0,
      heap_size: 0,
      next: 0,
    }
  }

  fn init(&mut self, heap_start: usize, heap_size: usize) {
    self.heap_start = heap_start;
    self.heap_size = heap_size;
    self.next = heap_start;
  }

  /// Allocates a block of memory with the given size and alignment.
  fn allocate(&mut self, size: usize, align: usize) -> Option<*mut u8> {
    let alloc_start = align_up(self.next, align);
    let alloc_end = alloc_start.saturating_add(size);

    if alloc_end <= self.heap_start + self.heap_size {
      self.next = alloc_end;
      Some(alloc_start as *mut u8)
    }
    else {
      None
    }
  }
}

/// Align downwards. Returns the greatest x with alignment `align` so that x <= addr. The alignment
/// must be a power of 2.
pub fn align_down(addr: usize, align: usize) -> usize {
  if align.is_power_of_two() {
    addr & !(align - 1)
  }
  else if align == 0 {
    addr
  }
  else {
    panic!("align_down - `align` must be a power of 2");
  }
}

/// Align upwards. Returns the smallest x with alignment `align` so that x >= addr. The alignment
/// must be a power of 2.
pub fn align_up(addr: usize, align: usize) -> usize {
  align_down(addr + align - 1, align)
}

#[no_mangle]
pub extern fn __rust_allocate(size: usize, align: usize) -> *mut u8 {
  unsafe {
    BUMP_ALLOCATOR.allocate(size, align).expect("out of memory")
  }
}

#[no_mangle]
pub extern fn __rust_deallocate(_ptr: *mut u8, _size: usize, _align: usize) {
  // leak it...
}

#[no_mangle]
pub extern fn __rust_usable_size(size: usize, _align: usize) -> usize {
  size
}

#[no_mangle]
pub extern fn __rust_reallocate_inplace(_ptr: *mut u8, size: usize, _new_size: usize, _align: usize) -> usize {
  size
}

#[no_mangle]
pub extern fn __rust_reallocate(ptr: *mut u8, size: usize, new_size: usize, align: usize) -> *mut u8 {
  use core::{ptr, cmp};

  let new_ptr = __rust_allocate(new_size, align);
  unsafe { ptr::copy(ptr, new_ptr, cmp::min(size, new_size)) };
  __rust_deallocate(ptr, size, align);
  new_ptr
}