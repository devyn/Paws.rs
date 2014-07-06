//! Locals make up the in-`Execution` data storage objects.

use object::*;
use machine::Machine;

use std::io::IoResult;

#[cfg(test)]
mod tests;

/// Very similar to a `Thing`, but also contains a 'name' symbol (usually
/// representing the string "locals") to which, if its receiver is invoked with
/// it, it will respond with the `Locals` object itself.
#[deriving(Clone)]
pub struct Locals {
  name: ObjectRef,
  meta: Meta
}

impl Locals {
  /// Creates a new `Locals` with the given name, and no members.
  pub fn new(name: ObjectRef) -> Locals {
    Locals {
      name: name,
      meta: Meta::with_receiver(locals_receiver)
    }
  }
}

impl Object for Locals {
  fn fmt_paws(&self, writer: &mut Writer) -> IoResult<()> {
    write!(writer, "Locals")
  }

  fn meta<'a>(&'a self) -> &'a Meta {
    &self.meta
  }

  fn meta_mut<'a>(&'a mut self) -> &'a mut Meta {
    &mut self.meta
  }
}

/// Returns the `subject` if the `message` is the `subject`'s name.
///
/// Otherwise, compares pair-wise, like `lookup_receiver`.
#[allow(unused_variable)]
pub fn locals_receiver(machine: &Machine, params: Params) -> Reaction {
  let lookup_result = {
    match params.subject.lock().try_cast::<Locals>() {
      Ok(subject) =>
        if params.message.eq_as_symbol(&subject.deref().name) {
          Some(params.subject.clone())
        } else {
          subject.deref().meta().members.lookup_pair(&params.message)
        },
      Err(subject) =>
        subject.deref().meta().members.lookup_pair(&params.message)
    }
  };

  debug!("{} <locals_receiver> {} => {}",
    params.subject, params.message, lookup_result);

  match lookup_result {
    Some(value) =>
      React(params.caller.clone(), value),
    None =>
      Yield
  }
}
