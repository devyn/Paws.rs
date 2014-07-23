use super::Reactor;
use super::realize;

use machine::Machine;

use object::ObjectRef;

use std::collections::{Deque, RingBuf};
use std::sync::Semaphore;
use std::mem::replace;

/// A reactor that executes without attempting any parallelism whatsoever.
///
/// It does not spawn its own task, so ensure that you either register a stall
/// handler to `stop()` it or run it in another task if you don't want to
/// potentially hang forever on `run()`.
pub struct SerialReactor {
  alive:          bool,
  stagings:       RingBuf<(ObjectRef, ObjectRef)>,
  stall_handlers: Vec<proc (&mut Reactor)>,
  machine:        Machine
}

impl SerialReactor {
  /// Creates a new SerialReactor with an empty queue and no stall handlers for
  /// the given Machine.
  pub fn new(machine: Machine) -> SerialReactor {
    SerialReactor {
      alive:          true,
      stagings:       RingBuf::new(),
      stall_handlers: Vec::new(),
      machine:        machine
    }
  }

  /// Returns `true` until `stop()` is called.
  pub fn is_alive(&self) -> bool {
    self.alive
  }

  /// Takes a single staging off the internal queue and reacts it, realizing the
  /// execution and response.
  ///
  /// Returns `false` if the reactor is no longer alive, or the queue is empty.
  pub fn step(&mut self) -> bool {
    if self.alive {
      match self.stagings.pop_front() {
        Some((execution, response)) => {
          realize(self, execution, response);
          true
        },
        None => false
      }
    } else {
      false
    }
  }

  /// Immediately invokes the reactor's stall handlers.
  pub fn stall(&mut self) {
    let stall_handlers = replace(&mut self.stall_handlers, Vec::new());

    for handler in stall_handlers.move_iter() {
      handler(&mut *self);
    }
  }

  /// Steps repeatedly until there is no more work to be done. Calls stall
  /// handlers automatically, and continues if they produce work.
  ///
  /// If there is no more work to be done and the reactor is still alive, the
  /// task will hang forever.
  pub fn run(&mut self) {
    loop {
      // Keep stepping until we either die or run out of work.
      while self.step() && self.alive { }

      // If we are no longer alive, we have to stop.
      if !self.alive { break }

      // Otherwise, try to call stall handlers to hopefully get more work or
      // stop.
      self.stall();

      // If our stall handlers didn't produce any work, or stopped us, we have
      // to exit the loop.
      if !self.alive || !self.step() { break }
    }

    // If we're still alive, we should hang: there's nothing more to be done,
    // and we're supposed to seem like we're still doing something.
    if self.alive {
      // Easiest way to block forever, I think.
      Semaphore::new(0).acquire();
    }
  }
}

impl Reactor for SerialReactor {
  fn stage(&mut self, execution: ObjectRef, response: ObjectRef) {
    if self.alive {
      self.stagings.push_back((execution, response));
    }
  }

  fn on_stall(&mut self, handler: proc (&mut Reactor)) {
    if self.alive {
      self.stall_handlers.push(handler);
    }
  }

  fn stop(&mut self) {
    self.alive = false;

    // Exhaust the stagings queue
    while self.stagings.pop_front().is_some() { }

    // Drop the stall handlers
    self.stall_handlers.truncate(0);
  }

  fn machine(&self) -> &Machine {
    &self.machine
  }
}
