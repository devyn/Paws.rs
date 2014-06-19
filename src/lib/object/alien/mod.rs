//! Aliens are similar to Executions but with native, opaque functionality.

use object::*;
use machine::*;

use std::any::*;
use std::io::IoResult;

#[cfg(test)]
mod tests;

/// An Alien Object behaves just like any Execution, but the behavior of a
/// combination against it is defined by the Routine type it contains, i.e.
/// native functionality.
///
/// Not only are Aliens Paws.rs' foreign function interface, but without them,
/// Paws.rs would be completely useless, as all of the initial useful operations
/// in Paws rely on a few native bootstrap routines.
///
/// Most operations which handle Executions should be capable of transparently
/// handling Aliens as well.
pub struct Alien {
  pub  routine: ~Routine,
  priv meta:    Meta
}

impl Alien {
  /// Construct an Alien around a given Routine.
  pub fn new(routine: ~Routine) -> Alien {
    Alien {
      routine: routine,
      meta:    Meta::new()
    }
  }

  /// Handle combination of this Alien (as the subject) with a given caller and
  /// message, in the context of a Machine.
  ///
  /// This really just hands execution off to the Alien's internal Routine, so
  /// see the docs for the `combine()` function there for the nitty-gritty
  /// details.
  pub fn combine(&mut self, machine: &mut Machine, caller: ObjectRef,
                 message: ObjectRef) {
    self.routine.combine(machine, caller, &mut self.meta, message)
  }
}

impl Object for Alien {
  fn fmt_paws(&self, writer: &mut Writer, machine: &Machine) -> IoResult<()> {
    try!(write!(writer, "Alien \\{ routine: "));

    try!(self.routine.fmt_paws(writer, machine));

    try!(write!(writer, " \\}"));

    Ok(())
  }

  fn meta<'a>(&'a self) -> &'a Meta {
    &self.meta
  }

  fn meta_mut<'a>(&'a mut self) -> &'a mut Meta {
    &mut self.meta
  }
}

impl Clone for Alien {
  fn clone(&self) -> Alien {
    Alien {
      routine: self.routine.to_owned(),
      meta:    self.meta.clone()
    }
  }
}

/// Handles the actual logic for an Alien, so different Aliens can have
/// different functionality.
///
/// We'd probably prefer to use a closure of some sort, but this is not bad, and
/// Rust doesn't really have the kind of closure we'd need for this anyway. I
/// have a feeling this is more flexible, too, and perhaps has less overhead.
pub trait Routine: Clone + Send + Share {
  /// Performs the routine-defined combination action on the given objects in
  /// the context of a machine.
  ///
  /// # Arguments
  ///
  /// * `machine`: The machine in which to execute. Mutable reference in case
  /// the routine wants to add entries to the queue or whatever.
  ///
  /// * `caller`: Reference to the execution that called us.
  ///
  /// * `subject_meta`: Metadata on the actual Object that was combined into (as
  /// the subject of the combination) that handed off to this Routine, in case
  /// we want to read/write it.
  ///
  /// * `message`: Reference to the Object that was combined with us (the
  /// message of the combination).
  fn combine(&mut self, machine: &mut Machine, caller: ObjectRef,
             subject_meta: &mut Meta, message: ObjectRef);

  /// Standard Paws formatting method; see Object.
  fn fmt_paws(&self, writer: &mut Writer, machine: &Machine) -> IoResult<()>;

  /// Clone inner data and return a new routine wrapped in an owned trait
  /// object.
  ///
  /// Contractually obligated to be of the same actual type, but there's no
  /// compile-time type safety here for obvious reasons, so "evil" things can
  /// happen.
  fn to_owned(&self) -> ~Routine {
    ~self.clone() as ~Routine
  }

  /// Converts a Routine trait object to an Any trait object.
  ///
  /// This is useful for attempting to convert an Routine trait object into its
  /// original type.
  fn as_any<'a>(&'a self) -> &'a Any {
    self as &Any
  }

  /// Same as `as_any()` but for a mutable ref.
  fn as_any_mut<'a>(&'a mut self) -> &'a mut Any {
    self as &mut Any
  }
}
