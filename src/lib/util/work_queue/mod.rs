//! Thread-safe work queue implementation.
//!
//! Keeps track of the number of workers currently doing work via guards, and
//! attempts to wake up a worker in the event of a stall, where no workers are
//! doing work and the queue is empty, so no work can be produced. This is
//! temporary.

use std::collections::RingBuf;
use std::collections::Deque;

use std::sync::Mutex;

#[cfg(test)]
mod tests;

struct QueueRoot<T> {
  alive:          bool,

  stall_notified: bool,
  workers:        uint,

  ring_buf:       RingBuf<T>
}

/// A blocking FIFO queue based on a ring buffer and a mutex, that tracks the
/// number of workers that currently have work and notifies in the event of a
/// stall.
pub struct WorkQueue<T> {
  root: Mutex<QueueRoot<T>>
}

impl<T: 'static+Send+Share> WorkQueue<T> {
  /// Creates a new queue.
  pub fn new() -> WorkQueue<T> {
    WorkQueue {
      root: Mutex::new(
        QueueRoot {
          alive:          true,
          stall_notified: false,
          workers:        0,
          ring_buf:       RingBuf::with_capacity(1)
        }
      )
    }
  }

  /// Pushes a message onto the queue.
  ///
  /// Wakes up one blocked task to notify it that data is available, if present.
  pub fn push(&self, message: T) {
    let mut root = self.root.lock();

    if !root.alive { return }

    root.ring_buf.push_back(message);

    root.stall_notified = false;

    root.cond.signal();
  }

  /// Takes the first message out of the queue. If the queue is empty, this
  /// function will block until either the queue has ended, stalled, or is no
  /// longer empty.
  pub fn shift<'a>(&'a self) -> ShiftResult<'a, T> {
    let mut root = self.root.lock();

    loop {
      if !root.alive { return Ended }

      if !root.ring_buf.is_empty() { break }

      if root.workers == 0 && !root.stall_notified {
        // No work available + no workers = stall, but only if we haven't
        // already notified (we don't want to generate a 'Stalled' message
        // twice)
        root.stall_notified = true;
        return Stalled

      } else {
        root.cond.wait()
      }
    }

    root.workers += 1;

    Work(WorkGuard::new(root.ring_buf.pop_front().unwrap(), self))
  }

  /// Forcibly ends the queue. All messages remaining in the queue are dropped
  /// immediately, further `push()` calls will be ignored, and `shift()` calls
  /// will always return `None`.
  ///
  /// Additionally, it wakes up all tasks that were waiting for messages,
  /// notifying them immediately.
  pub fn end(&self) {
    let mut root = self.root.lock();

    root.alive    = false;
    root.ring_buf = RingBuf::new();

    root.cond.broadcast();
  }

  /// Decrements the 'workers' count. Won't go below zero. Wakes up other tasks
  /// if we hit zero and the queue's still alive.
  fn drop_worker(&self) {
    let mut root = self.root.lock();
    
    if root.workers > 0 {
      root.workers -= 1;

      if root.workers == 0 && root.alive {
        root.cond.signal();
      }
    }
  }
}

/// Represents the result of a `shift()`.
pub enum ShiftResult<'a, T> {
  /// Work was acquired from the queue.
  Work(WorkGuard<'a, T>),

  /// The queue has stalled; no one is doing work on it and it's empty.
  Stalled,

  /// The queue has been explicitly ended.
  Ended
}

/// Wraps the work such that, when dropped, it notifies the queue.
///
/// It is important that this isn't dropped while work is still being done
/// related to something that came from this queue. This allows the queue to
/// detect stalls.
pub struct WorkGuard<'a, T> {
  work:  T,
  queue: &'a WorkQueue<T>
}

impl<'a, T: 'static+Send+Share> WorkGuard<'a, T> {
  fn new(work: T, queue: &'a WorkQueue<T>) -> WorkGuard<'a, T> {
    WorkGuard { work: work, queue: queue }
  }
}

impl<'a, T: 'static+Send+Share> Deref<T> for WorkGuard<'a, T> {
  fn deref<'b>(&'b self) -> &'b T {
    &self.work
  }
}

#[unsafe_destructor]
impl<'a, T: 'static+Send+Share> Drop for WorkGuard<'a, T> {
  fn drop(&mut self) {
    self.queue.drop_worker();
  }
}
