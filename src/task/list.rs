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

#[cfg(test)]
mod tests {
  use super::TaskQueue;
  use super::super::TaskControl;
  use alloc::boxed::Box;

  #[test]
  fn empty_dequeue() {
    let mut list = TaskQueue::new();

    assert!(list.dequeue().is_none());
  }

  #[test]
  fn basics() {
    let mut list = TaskQueue::new();

    let t1 = Box::new(TaskControl::uninitialized("1"));
    let t2 = Box::new(TaskControl::uninitialized("2"));
    let t3 = Box::new(TaskControl::uninitialized("3"));
    let t4 = Box::new(TaskControl::uninitialized("4"));
    let t5 = Box::new(TaskControl::uninitialized("5"));
    let t6 = Box::new(TaskControl::uninitialized("6"));
    let t7 = Box::new(TaskControl::uninitialized("7"));

    let pt1: *const _ = &*t1;
    let pt2: *const _ = &*t2;
    let pt3: *const _ = &*t3;
    let pt4: *const _ = &*t4;
    let pt5: *const _ = &*t5;
    let pt6: *const _ = &*t6;
    let pt7: *const _ = &*t7;

    // Populate list
    list.enqueue(t1);
    list.enqueue(t2);
    list.enqueue(t3);

    // Check normal removal
    assert_eq!(&*list.dequeue().unwrap() as *const _, pt1);
    assert_eq!(&*list.dequeue().unwrap() as *const _, pt2);

    // Push some more just to make sure nothing's corrupted
    list.enqueue(t4);
    list.enqueue(t5);

    // Check normal removal
    assert_eq!(&*list.dequeue().unwrap() as *const _, pt3);
    assert_eq!(&*list.dequeue().unwrap() as *const _, pt4);

    // Check exhaustion
    assert_eq!(&*list.dequeue().unwrap() as *const _, pt5);
    assert!(list.dequeue().is_none());

    // Check the exhaustion case fixed the pointer right
    list.enqueue(t6);
    list.enqueue(t7);

    // Check normal removal
    assert_eq!(&*list.dequeue().unwrap() as *const _, pt6);
    assert_eq!(&*list.dequeue().unwrap() as *const _, pt7);
    assert!(list.dequeue().is_none());
  }
}
