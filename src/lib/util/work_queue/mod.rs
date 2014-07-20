//! Thread-safe work queue implementation.
//!
//! Keeps track of the number of workers currently doing work via guards, and
//! attempts to wake up a worker in the event of a stall, where no workers are
//! doing work and the queue is empty, so no work can be produced. This is
//! temporary.

use std::sync::Arc;
use std::sync::Mutex;

use std::sync::atomics::{AtomicBool, AtomicUint, SeqCst};

#[cfg(test)]
mod tests;

/// A blocking FIFO queue based on a channel and a mutex, that tracks the number
/// of workers that currently have work and notifies in the event of a stall.
pub struct WorkQueue<T> {
  workers: uint,
  tx:      Sender<T>,
  rx:      Arc<Mutex<Receiver<T>>>,
  alive:   Arc<AtomicBool>,
  waiting: Arc<AtomicUint>
}

impl<T: 'static+Send+Share> WorkQueue<T> {
  /// Creates a new queue for the specified number of workers.
  pub fn new(workers: uint) -> WorkQueue<T> {
    let (tx, rx) = channel::<T>();

    WorkQueue {
      workers: workers,
      tx:      tx,
      rx:      Arc::new(Mutex::new(rx)),
      alive:   Arc::new(AtomicBool::new(true)),
      waiting: Arc::new(AtomicUint::new(0))
    }
  }

  /// Pushes a message onto the queue.
  pub fn push(&self, message: T) {
    if self.alive.load(SeqCst) {
      self.tx.send(message);

      if self.waiting.load(SeqCst) > 0 {
        self.rx.lock().cond.signal();
      }
    }
  }

  /// Takes the first message out of the queue. If the queue is empty, this
  /// function will block until either the queue has ended, stalled, or is no
  /// longer empty.
  pub fn shift(&self) -> ShiftResult<T> {
    let mut waited = false;

    let mut rx = self.rx.lock();

    loop {
      if !self.alive.load(SeqCst) {
        if waited {
          self.waiting.fetch_sub(1, SeqCst);
        }

        return Ended;
      }

      match rx.try_recv().ok() {
        Some(work) => {
          if waited {
            self.waiting.fetch_sub(1, SeqCst);
          }

          return Work(work);
        },
        None => {
          if !waited {
            let waiting = self.waiting.fetch_add(1, SeqCst) + 1;

            debug!("waiting: {}", waiting);
              
            if waiting == self.workers {
              self.waiting.fetch_sub(1, SeqCst);
              return Stalled;
            }
            waited = true;
          }

          rx.cond.wait();
        }
      }
    }
  }

  /// Forcibly ends the queue. Further calls to `shift()` will return `Ended`,
  /// and currently waiting calls to `shift()` will be woken up to return
  /// `Ended` as well.
  pub fn end(&self) {
    self.alive.store(false, SeqCst);

    self.rx.lock().cond.broadcast();
  }
}

impl<T: 'static+Send+Share> Clone for WorkQueue<T> {
  fn clone(&self) -> WorkQueue<T> {
    WorkQueue {
      workers: self.workers,
      tx:      self.tx.clone(),
      rx:      self.rx.clone(),
      alive:   self.alive.clone(),
      waiting: self.waiting.clone()
    }
  }
}

/// Represents the result of a `shift()`.
pub enum ShiftResult<T> {
  /// Work was acquired from the queue.
  Work(T),

  /// The queue has stalled; no one is doing work on it and it's empty.
  Stalled,

  /// The queue has been explicitly ended.
  Ended
}
