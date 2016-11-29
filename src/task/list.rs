use super::TaskControl;
use ::alloc::boxed::Box;

pub struct TaskQueue {
  head: Option<Box<TaskControl>>,
  tail: *mut TaskControl,
}

impl TaskQueue {
  pub const fn new() -> Self {
    TaskQueue { 
      head: None,
      tail: ::core::ptr::null_mut(),
    }
  }

  pub fn enqueue(&mut self, elem: Box<TaskControl>) {
    let mut new_tail = elem;

    let raw_tail: *mut _ = &mut *new_tail;

    if !self.tail.is_null() {
      unsafe {
        (*self.tail).next = Some(new_tail);
      }
    }
    else {
      self.head = Some(new_tail);
    }

    self.tail = raw_tail;
  }

  pub fn dequeue(&mut self) -> Option<Box<TaskControl>> {
    match self.head.take() {
      Some(mut head) => {
        self.head = head.next.take();

        if self.head.is_none() {
          self.tail = ::core::ptr::null_mut();
        }

        Some(head)
      },
      None => None,
    }
  }
}
