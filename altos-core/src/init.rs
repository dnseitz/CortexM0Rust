// init.rs
// AltOSRust
//
// Created by Daniel Seitz on 12/9/16

pub fn init_heap(heap_start: usize, heap_size: usize) {
  #[cfg(not(test))]
  ::bump_allocator::init_heap(heap_start, heap_size);
}
