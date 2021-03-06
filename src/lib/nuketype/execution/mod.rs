//! Executions are like mutable, contained coroutines.
//!
//! From the [specification](http://ell.io/spec) itself:
//!
//! > Although they are similar to the more traditional "continuations" from
//! > programming-language theory, our executions are *not* static. One does not
//! > simply take an execution, and then have a handle to resume execution at
//! > the point it was taken indefinitely. When a particular execution-object is
//! > used to resume execution at that point, the object itself "moves forward"
//! > **with** the procedure's execution.

use script::*;

use object::{ObjectRef, Meta, Params};

use nuketype::{Nuketype, Locals};

use machine::Machine;
use machine::reactor::{Reactor, Combination};
use machine::reactor::{Combinable, FromSelf, FromLocals, From};

use util::clone;

use std::io::IoResult;
use std::sync::Arc;

#[cfg(test)]
mod tests;

/// Implements an Execution as a reference to a Script, a program counter, as
/// well as a stack for evaluating subexpressions.
///
/// **Note:** When boxing this nuketype up, make sure to set the receiver to
/// `stage_receiver` and set up a locals object. `Execution::create()` does this
/// automatically, so prefer that to `Execution::new()` if possible.
#[deriving(Clone)]
pub struct Execution {
  root:     Arc<Script>,
  pc:       uint,
  stack:    Vec<Combinable>
}

impl Execution {
  /// Creates a new Execution with the given Script as its root.
  pub fn new(root: Script) -> Execution {
    Execution {
      root:     Arc::new(root),
      pc:       0,
      stack:    Vec::new()
    }
  }

  /// Boxes up an Execution into an object with its receiver set to
  /// `stage_receiver` and a new, empty `locals` object.
  pub fn create(machine: &Machine, root: Script) -> ObjectRef {
    let mut meta = Meta::with_receiver(stage_receiver);

    meta.members.push_pair_to_child(
      machine.locals_sym.clone(),
      Locals::empty(machine.locals_sym.clone()));

    ObjectRef::store(box Execution::new(root), meta)
  }

  /// Returns the "root" Script of the Execution, which the Execution's internal
  /// program counter ("pc") is based on.
  pub fn root<'a>(&'a self) -> &'a Script {
    &*self.root
  }

  /// Advances the Execution, first pushing `response` onto the stack, moving
  /// its program counter forward and evaluating instructions, ending with
  /// either the execution of a Combine instruction or completion.
  pub fn advance(&mut self, response: ObjectRef) -> Option<Combination> {
    let Script(ref instructions) = *self.root;

    if self.pc < instructions.len() {
      self.stack.push(From(response));
    }

    while self.pc < instructions.len() {
      let instruction = &instructions[self.pc];

      self.pc += 1;

      debug!("advance: {} on (stack: {})", instruction, self.stack);

      match *instruction {
        PushLocals =>
          self.stack.push(FromLocals),

        PushSelf =>
          self.stack.push(FromSelf),

        Push(ref object) =>
          self.stack.push(From(object.clone())),

        Combine => {
          let message = self.stack.pop().expect("stack too small");
          let subject = self.stack.pop().expect("stack too small");

          return Some(Combination {
            subject: subject,
            message: message
          })
        },

        Discard =>
          { self.stack.pop(); }
      }
    }

    None // default (completion)
  }
}

impl Nuketype for Execution {
  fn fmt_paws(&self, writer: &mut Writer) -> IoResult<()> {
    let Script(ref instructions) = *self.root;

    write!(writer, "Execution {{ pc: {} => {}, stack: {} }}",
      self.pc, instructions[self.pc], self.stack)
  }
}

/// A receiver that first ensures the subject is stageable, clones it, and then
/// enqueues the clone with the message.
pub fn stage_receiver(reactor: &mut Reactor, params: Params) {
  match clone::stageable(&params.subject, reactor.machine()) {
    Some(clone) => {
      debug!("stage_receiver: {} cloned to {} <-- {}",
             params.subject, clone, params.message);

      reactor.stage(clone, params.message.clone());
    },

    None =>
      warn!(concat!("stage_receiver failed: {} <-- {}, subject is neither an",
                    " execution nor an alien"),
            params.subject, params.message)
  }
}
