//! Thread-safe queue implementation.

use std::mem;

use sync::Mutex;

#[cfg(test)]
mod tests;

struct QueueRoot<T> {
  alive: bool,
  first: Box<QueueNode<T>>,
  last:  *mut QueueNode<T>
}

enum QueueNode<T> {
  Nil,
  Cons(T, Box<QueueNode<T>>)
}

/// A somewhat efficient blocking FIFO queue based on a linked list and a mutex.
pub struct Queue<T> {
  root: Mutex<QueueRoot<T>>
}

impl<T: 'static+Send+Share> Queue<T> {
  /// Creates a new queue.
  pub fn new() -> Queue<T> {
    Queue {
      root: Mutex::new({
        let mut first = box Nil;

        QueueRoot {
          alive: true,
          last:  &mut *first as *mut QueueNode<T>,
          first: first
        }
      })
    }
  }

  /// Pushes a message onto the queue.
  ///
  /// Wakes up one blocked task to notify it that data is available, if present.
  pub fn push(&self, message: T) {
    let mut root = self.root.lock();

    if !root.alive { return }

    let last_mut: &mut QueueNode<T> = unsafe { &mut *root.last };

    let mut next = box Nil;

    root.last = &mut *next as *mut QueueNode<T>;
    *last_mut = Cons(message, next);

    root.cond.signal();
  }

  /// Takes the first message out of the queue. If the queue is empty, this
  /// function will block until either the queue has ended or is no longer
  /// empty.
  pub fn shift(&self) -> Option<T> {
    let mut root = self.root.lock();

    loop {
      if !root.alive { return None }

      match root.first {
        box Cons(..) => {
          // This hack avoids memory allocation when we're moving things around.
          //
          // TODO: need a better way to do this. Surely there must be one.
          let first = mem::replace(&mut root.first, unsafe { mem::zeroed() });

          match first {
            box Cons(message, next) => {
              let invalid = mem::replace(&mut root.first, next);

              // If we don't do this, Rust will try to drop() some invalid data.
              unsafe { mem::forget(invalid); }

              return Some(message);
            },
            _ => unreachable!()
          }
        },
        box Nil =>
          root.cond.wait()
      }
    }
  }

  /// Forcibly ends the queue. All messages remaining in the queue are dropped
  /// immediately, further `push()` calls will be ignored, and `shift()` calls
  /// will always return `None`.
  ///
  /// Additionally, it wakes up all tasks that were waiting for messages,
  /// notifying them immediately.
  pub fn end(&self) {
    let mut root = self.root.lock();

    root.alive = false;
    root.first = unsafe { mem::zeroed() };
    root.last  = unsafe { mem::zeroed() };

    root.cond.broadcast();
  }

  /// Creates an iterator over the messages in this queue, ending when the queue
  /// is explicitly `end()`ed.
  pub fn iter<'a>(&'a self) -> Messages<'a, T> {
    Messages { queue: self }
  }
}

/// Iterator over messages in a queue.
pub struct Messages<'a, T> {
  queue: &'a Queue<T>
}

impl<'a, T:'static+Send+Share> Iterator<T> for Messages<'a, T> {
  fn next(&mut self) -> Option<T> {
    self.queue.shift()
  }
}
