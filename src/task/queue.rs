
use super::TaskControl;
use ::alloc::boxed::Box;
use ::core::ptr;

pub struct TaskQueue {
  rear: *mut TaskControl,
}

/// A task queue to keep track of ready tasks. It is implemented as an unsafe CLL using raw
/// pointers to save on memory usage as we have very little RAM and deallocating/reallocating 
/// Boxes all over the place would make many holes in our memory structure. 
impl TaskQueue {
  pub const fn new() -> Self {
    TaskQueue { rear: ptr::null_mut() }
  }

  pub fn enqueue(&mut self, elem: Box<TaskControl>) {
    let elem = Box::into_raw(elem);
    if self.rear.is_null() {
      unsafe {
        (*elem).next = elem;
      }
    }
    else {
      unsafe {
        (*elem).next = (*self.rear).next;
        (*self.rear).next = elem;
      }
    }
    self.rear = elem;
  }

  pub fn dequeue(&mut self) -> Option<&TaskControl> {
    if self.rear.is_null() {
      None
    }
    else {
      let front = unsafe { (*self.rear).next };
      if self.rear == front as *mut TaskControl {
        self.rear = ptr::null_mut(); 
      }
      else {
        unsafe {
          (*self.rear).next = (*front).next;
        }
      }
      unsafe {
        Some(&*front)
      }
    }
  }
}

