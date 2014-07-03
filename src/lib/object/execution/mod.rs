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

use script::*;
use object::*;
use machine::{Machine, Combination};

#[cfg(test)]
mod tests;

/// Implements an Execution as a reference to a Script, a program counter, as
/// well as a stack for evaluating subexpressions.
#[deriving(Clone)]
pub struct Execution {
  root:     Script,
  pristine: bool,
  pc:       Vec<uint>,
  stack:    Vec<Option<ObjectRef>>,
  meta:     Meta
}

impl Execution {
  /// Creates a new Execution with the given Script as its root.
  pub fn new(root: Script) -> Execution {
    Execution {
      root:     root,
      pristine: true,
      pc:       Vec::new(),
      stack:    Vec::new(),
      meta:     Meta::new()
    }
  }

  /// Returns the "root" Script of the Execution, which the Execution's internal
  /// program counter ("pc") is based on.
  pub fn root<'a>(&'a self) -> &'a Script {
    &self.root
  }

  /// Advances the Execution, producing a Combination to be staged if the
  /// Execution is not at the end of its root script.
  ///
  /// # Arguments
  ///
  /// - **self_ref**: The reference to this Execution. Used for interpreting an
  /// empty expression; `()` results in a reference to this Execution.
  /// - **response**: The object this Execution is being sent.
  pub fn advance(&mut self, self_ref: ObjectRef,
                 response: ObjectRef) -> Option<Combination> {

    // If the Execution is pristine, we need to disregard the response. This is
    // just to remind us.
    let was_pristine = self.pristine;

    if !self.pristine && self.stack.is_empty() && self.pc.is_empty() {
      // This Execution has been completed; no Combination can be produced.
      return None;
    } else if self.pristine {
      // Execution is pristine and needs to be initialized. Set program counter
      // to the first node.
      self.pristine = false;
      self.pc.push(0);
    } else {
      *self.pc.mut_last().unwrap() += 1;
    }

    match node_at_pc(&self.root, self.pc.as_slice()) {
      None => {
        // If there was no Node after the original pc, the current Node is the
        // enclosing Expression.
        self.pc.pop();

        // If the pc is empty, this is the end; there is no Combination to be
        // produced.
        if self.pc.is_empty() {
          None
        } else {
          // Don't need to worry about disregarding the response, since this
          // can't possibly happen if the execution was pristine.
          Some(Combination {
            subject: self.stack.pop().unwrap(),
            message: response
          })
        }
      },

      Some(node_at_pc) => {
        // Points to the current node while we iterate through expression nodes
        // until we manage to get an object of some sort.
        let mut current = node_at_pc;

        // Counts iterations of the loop. The response gets pushed onto the
        // stack and consumed on the first iteration.
        let mut iterations = 0u;

        // Contains the response if it is not yet consumed by the first
        // iteration of #4. The response is consumed if an ExpressionNode is
        // encountered.
        let mut response_if_unconsumed =
          if !was_pristine {
            Some(response)
          } else {
            // If the execution was pristine, we need to disregard the response
            // and just combine against locals anyway.
            None
          };

        // The resulting message of the combination.
        let mut resulting_message: ObjectRef;

        // Descends into ExpressionNodes until we get to either an empty
        // ExpressionNode (which has a special meaning) or an ObjectNode.
        loop {
          iterations += 1;

          match current {
            &ExpressionNode(ref nodes) =>
              if nodes.is_empty() {
                // An empty ExpressionNode is a special case that refers to this
                // own Execution. Thus this is treated the same as if we had
                // found an ObjectNode, but the resulting message is self_ref.
                resulting_message = self_ref;
                break;
              } else {
                // On the first ExpressionNode iteration through this loop, the
                // response is consumed and pushed onto the stack for later.
                if iterations == 1 {
                  self.stack.push(response_if_unconsumed);
                } else {
                  self.stack.push(None);
                }

                response_if_unconsumed = None;

                // Descend into the ExpressionNode.
                self.pc.push(0);
                current = nodes.get(0);
              },

            &ObjectNode(ref object_ref) => {
              // Should we encounter an ObjectNode, the object it contains is
              // the message we combine with.
              resulting_message = object_ref.clone();
              break;
            }
          }
        }

        // If we encountered any ExpressionNodes on our path to an object, the
        // response_if_unconsumed is None, so this is a Combination against the
        // Execution's locals. Otherwise, it's a Combination against the
        // response given to this function.
        Some(Combination {
          subject: response_if_unconsumed,
          message: resulting_message
        })
      }
    }
  }
}

fn node_at_pc<'a>(script: &'a Script, pc: &[uint]) -> Option<&'a Node> {

  let &Script(ref inner_nodes) = script;

  let mut nodes = inner_nodes;

  for &i in pc.init().iter() {
    match nodes.get(i) {
      &ExpressionNode(ref inner_nodes) => {
        nodes = inner_nodes;
      },
      _ => fail!("Expected all pc positions except last one to point to \
                  ExpressionNodes.")
    }
  }

  let i = *pc.last().unwrap();

  if i < nodes.len() {
    Some(nodes.get(i))
  } else {
    None
  }
}

impl Object for Execution {
  fn fmt_paws(&self, writer: &mut Writer) -> IoResult<()> {
    try!(write!(writer, "Execution {{ root: "));

    try!(self.root.fmt_paws(writer));

    try!(write!(writer, ", pristine: {}, pc: {}, stack: [",
      self.pristine, self.pc));

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

  fn default_receiver(&self) -> NativeReceiver {
    stage_receiver
  }
}

/// A receiver that simply tells the reactor to realize the subject (which
/// should be an Execution or an Alien) with the message as the response.
#[allow(unused_variable)]
pub fn stage_receiver(machine: &Machine, params: Params) -> Reaction {
  React(params.subject.clone(), params.message.clone())
}
