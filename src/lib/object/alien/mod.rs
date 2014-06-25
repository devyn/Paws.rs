//! Aliens are similar to Executions but with native, opaque functionality.

use object::*;
use machine::*;

use std::any::*;
use std::io::IoResult;

#[cfg(test)]
mod tests;

/// An Alien Object behaves just like any Execution, but the behavior of a
/// combination against it is defined by the Routine type it contains, i.e.
/// native functionality, as well as some alien-local data of an unknown type
/// (likely whatever the Routine expects).
///
/// Not only are Aliens Paws.rs' foreign function interface, but without them,
/// Paws.rs would be completely useless, as all of the initial useful operations
/// in Paws rely on a few native bootstrap routines.
///
/// Most operations which handle Executions should be capable of transparently
/// handling Aliens as well.
pub struct Alien {
  priv routine: Routine,
  priv data:    ~Any:'static,
  priv meta:    Meta
}

impl Alien {
  /// Construct an Alien around a given Routine.
  pub fn new(routine: Routine, data: ~Any:'static) -> Alien {
    Alien {
      routine: routine,
      data:    data,
      meta:    Meta::new()
    }
  }

  /// Give the Alien a response; similar to Execution advancement.
  pub fn advance(&mut self, machine: &mut Machine,
                 response: ObjectRef) -> Reaction {
    (self.routine)(machine, self, response)
  }
}

impl Object for Alien {
  #[allow(unused_variable)]
  fn fmt_paws(&self, writer: &mut Writer, machine: &Machine) -> IoResult<()> {
    write!(writer, "Alien")
  }

  fn meta<'a>(&'a self) -> &'a Meta {
    &self.meta
  }

  fn meta_mut<'a>(&'a mut self) -> &'a mut Meta {
    &mut self.meta
  }

  #[allow(unused_variable)]
  fn default_receiver<Alien>() -> NativeReceiver {
    |machine: &mut Machine, params: Params| -> Reaction {
      React(Stage(StageParams {
        execution: params.subject.clone(),
        response:  params.message.clone(),
        mask:      None
      }))
    }
  }
}

/// A function that implements the logic behind an Alien.
pub type Routine = fn(machine: &mut Machine, alien: &mut Alien,
                      response: ObjectRef) -> Reaction;
