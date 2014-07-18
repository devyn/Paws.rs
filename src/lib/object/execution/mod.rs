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

use std::io::IoResult;

use std::sync::Arc;

use script::*;
use object::*;
use machine::{Machine, Combination};

use util::clone;

#[cfg(test)]
mod tests;

/// Implements an Execution as a reference to a Script, a program counter, as
/// well as a stack for evaluating subexpressions.
#[deriving(Clone)]
pub struct Execution {
  root:     Arc<Script>,
  pc:       uint,
  stack:    Vec<Option<ObjectRef>>,
  meta:     Meta
}

impl Execution {
  /// Creates a new Execution with the given Script as its root.
  pub fn new(root: Script) -> Execution {
    Execution {
      root:     Arc::new(root),
      pc:       0,
      stack:    Vec::new(),
      meta:     Meta::with_receiver(stage_receiver)
    }
  }

  /// Returns the "root" Script of the Execution, which the Execution's internal
  /// program counter ("pc") is based on.
  pub fn root<'a>(&'a self) -> &'a Script {
    &*self.root
  }

  /// Advances the Execution, moving its program counter forward and evaluating
  /// instructions, ending with either the execution of a Combine instruction or
  /// completion.
  ///
  /// # Arguments
  ///
  /// - **machine**: The machine context in which this operation is running.
  ///
  /// - **self_ref**: The reference to this Execution. Used for interpreting an
  /// empty expression; `[]` results in a reference to this Execution.
  ///
  /// - **response**: The object this Execution is being sent.
  pub fn advance(&mut self,
                 self_ref: &ObjectRef,
                 response: ObjectRef)
                 -> Option<Combination> {
    let Script(ref instructions) = *self.root;

    if self.pc < instructions.len() {
      self.stack.push(Some(response));
    }

    while self.pc < instructions.len() {
      let instruction = &instructions[self.pc];

      self.pc += 1;

      debug!("advance {} {} (stack: {})", self_ref, instruction, self.stack);

      match *instruction {
        PushLocals =>
          self.stack.push(None),

        PushSelf =>
          self.stack.push(Some(self_ref.clone())),

        Push(ref object) =>
          self.stack.push(Some(object.clone())),

        Combine => {
          let message = self.stack.pop().expect("stack too small");
          let subject = self.stack.pop().expect("stack too small");

          return Some(Combination {
            subject: subject,
            message: message.expect("PushLocals result not allowed as message")
          })
        },

        Discard =>
          { self.stack.pop(); }
      }
    }

    None // default (completion)
  }
}

impl Object for Execution {
  fn fmt_paws(&self, writer: &mut Writer) -> IoResult<()> {
    let Script(ref instructions) = *self.root;

    try!(write!(writer, "Execution {{ pc: {} => {}, stack: [",
      self.pc, instructions[self.pc]));

    let mut stack_iter = self.stack.iter().peekable();

    loop {
      match stack_iter.next() {
        Some(&Some(ref object_ref)) =>
          try!(object_ref.lock().fmt_paws(writer)),

        Some(&None) =>
          try!(write!(writer, "NoObject")),

        None => break
      }

      if !stack_iter.is_empty() {
        try!(write!(writer, ", "));
      }
    }

    try!(write!(writer, "] }}"));

    Ok(())
  }

  fn meta<'a>(&'a self) -> &'a Meta {
    &self.meta
  }

  fn meta_mut<'a>(&'a mut self) -> &'a mut Meta {
    &mut self.meta
  }
}

/// A receiver that first ensures the subject is queueable, clones it, and then
/// enqueues the clone with the message.
#[allow(unused_variable)]
pub fn stage_receiver(machine: &Machine, params: Params) -> Reaction {
  match clone::queueable(&params.subject, machine) {
    Some(clone) => {
      debug!("stage_receiver: {} cloned to {} <-- {}",
             params.subject, clone, params.message);

      // React() would be unsafe; there may be other things depending on the
      // order
      machine.enqueue(clone, params.message.clone());
    },

    None =>
      warn!(concat!("stage_receiver failed: {} <-- {}, subject is neither an",
                    " execution nor an alien"),
            params.subject, params.message)
  }

  Yield
}
