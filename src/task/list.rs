use super::TaskControl;
use ::alloc::boxed::Box;

type Link<T> = Option<Box<Node<T>>>;

pub struct Queue<T> {
  head: Link<T>,
  tail: *mut Node<T>,
}

impl<T> Queue<T> {
  pub const fn new() -> Self {
    Queue { 
      head: None,
      tail: ::core::ptr::null_mut(),
    }
  }

  pub fn enqueue(&mut self, elem: T) {
    let mut new_tail = Box::new(Node::new(elem));

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

  pub fn dequeue(&mut self) -> Option<T> {
    match self.head.take() {
      Some(head) => {
        let head = *head;
        self.head = head.next;

        if self.head.is_none() {
          self.tail = ::core::ptr::null_mut();
        }

        Some(head.elem)
      },
      None => None,
    }
  }
}

struct Node<T> {
  elem: T,
  next: Link<T>,
}

impl<T> Node<T> {
  fn new(elem: T) -> Self {
    Node {
      elem: elem,
      next: None,
    }
  }
}

#[cfg(test)]
mod test {
  use super::Queue;
  #[test]
  fn basics() {
    let mut list = Queue::new();

    // Check empty list behaves right
    assert_eq!(list.dequeue(), None);

    // Populate list
    list.enqueue(1);
    list.enqueue(2);
    list.enqueue(3);

    // Check normal removal
    assert_eq!(list.dequeue(), Some(1));
    assert_eq!(list.dequeue(), Some(2));

    // Push some more just to make sure nothing's corrupted
    list.enqueue(4);
    list.enqueue(5);

    // Check normal removal
    assert_eq!(list.dequeue(), Some(3));
    assert_eq!(list.dequeue(), Some(4));

    // Check exhaustion
    assert_eq!(list.dequeue(), Some(5));
    assert_eq!(list.dequeue(), None);

    // Check the exhaustion case fixed the pointer right
    list.enqueue(6);
    list.enqueue(7);

    // Check normal removal
    assert_eq!(list.dequeue(), Some(6));
    assert_eq!(list.dequeue(), Some(7));
    assert_eq!(list.dequeue(), None);
  }
}
