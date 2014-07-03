//! Aliens are similar to Executions but with native, opaque functionality.

use object::*;
use object::execution::stage_receiver;
use machine::Machine;

use std::any::*;
use std::io::IoResult;
use std::mem::replace;

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
  /// A function to execute upon realization.
  pub routine: Routine,

  /// Routine-specific (non-generic) data. Often used to store multiple
  /// arguments when implementing the nuclear call-pattern.
  pub data:    Box<Any+'static+Send+Share>,

      meta:    Meta
}

impl Alien {
  /// Construct an Alien around a given `Routine`.
  pub fn new(routine: Routine, data: Box<Any+'static+Send+Share>) -> Alien {
    Alien {
      routine: routine,
      data:    data,
      meta:    Meta::new()
    }
  }

  /// Construct a call-pattern Alien which calls the given `CallPatternRoutine`
  /// once `n_args` arguments have been accepted.
  pub fn new_call_pattern(routine: CallPatternRoutine, n_args: uint) -> Alien {
    let call_pattern_data = box CallPatternData {
      caller:    None,
      args:      Vec::with_capacity(n_args),
      complete:  false,
      remaining: n_args,
      routine:   routine
    };

    Alien::new(call_pattern_alien_routine,
               call_pattern_data as Box<Any+'static+Send+Share>)
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
                       alien:    TypedRefGuard<'a, Alien>,
                       machine:  &Machine,
                       response: ObjectRef)
                       -> Reaction;

/// A function that implements a "call-pattern" style Alien.
///
/// This type of alien first accepts a 'caller', then (optionally) repeatedly
/// responds to that caller with itself to accumulate more arguments until it
/// reaches a pre-defined number, at which point the actual Alien logic is run.
///
/// After all of the arguments and caller have been accepted and a response has
/// been sent from the routine, the Alien no longer responds.
///
/// # Example
///
/// cPaws code (where `alien` is a call-pattern alien that takes a caller and 2
/// args):
///
///     alien() hello world
///
/// Assume this returns `hi`. Timeline:
///
///     alien <- ()    ... {caller = ()}        ... caller <- alien
///     alien <- hello ... {args   push(hello)} ... caller <- alien
///     alien <- world ... {args   push(world)} ...
///
///       call_pattern_routine(machine, caller = (), args = [hello, world])
///         -> React(caller, hi) ...
///
///     caller <- hi
pub type CallPatternRoutine = fn<'a>(
                                 machine: &Machine,
                                 caller:  ObjectRef,
                                 args:    &[ObjectRef])
                                 -> Reaction;

/// Internal state for call pattern wrapper.
struct CallPatternData {
  caller:    Option<ObjectRef>,
  args:      Vec<ObjectRef>,
  complete:  bool,
  remaining: uint,
  routine:   CallPatternRoutine
}

/// Function that performs call pattern wrapper.
fn call_pattern_alien_routine<'a>(
                              mut alien: TypedRefGuard<'a, Alien>,
                              machine:   &Machine,
                              response:  ObjectRef)
                              -> Reaction {

  let (caller, routine, args) = {
    // Do everything we need to do to data in here, so we can drop alien.
    let data = alien.data.as_mut::<CallPatternData>().unwrap();

    // Don't do anything if we're complete.
    if data.complete { return Yield }

    match data.caller {
      Some(_) => {
        data.args.push(response);
        data.remaining -= 1;
      },
      None => {
        data.caller = Some(response.clone());
      }
    }

    if data.remaining == 0 {
      let routine = data.routine;

      // Cheap way to deallocate all of the expensive stuff in data.
      let final = replace(data, CallPatternData {
        caller:    None,
        args:      Vec::new(),
        complete:  true,
        remaining: 0,
        routine:   routine
      });

      (final.caller.unwrap(), final.routine, Some(final.args))
    } else {
      (data.caller.get_ref().clone(), data.routine, None)
    }
  };

  match args {
    Some(args) => {
      // We have args, so we must be done.
      drop(alien);
      routine(machine, caller, args.as_slice())
    },
    None =>
      // Need more args!
      React(caller, alien.unlock().clone())
  }
}
