//! Aliens are similar to Executions but with native, opaque functionality.

use object::*;
use object::execution::stage_receiver;
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
  pub  routine: Routine,
  pub  data:    ~Any:'static+Send+Share,
  priv meta:    Meta
}

impl Alien {
  /// Construct an Alien around a given Routine.
  pub fn new(routine: Routine, data: ~Any:'static+Send+Share) -> Alien {
    Alien {
      routine: routine,
      data:    data,
      meta:    Meta::new()
    }
  }
}

impl Object for Alien {
  fn fmt_paws(&self, writer: &mut Writer) -> IoResult<()> {
    write!(writer, "Alien")
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

/// A function that implements the logic behind an Alien.
pub type Routine = fn <'a>(
                       alien: TypedRefGuard<'a, Alien>,
                       machine: &Machine,
                       response: ObjectRef)
                       -> Reaction;
