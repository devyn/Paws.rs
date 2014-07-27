//! Aliens are similar to Executions but with native, opaque functionality.

use object::{ObjectRef, TypedRefGuard};
use object::{Meta, Params, Tag};

use nuketype::Nuketype;

use machine::Reactor;

use std::any::{Any, AnyRefExt, AnyMutRefExt};
use std::io::IoResult;
use std::mem::replace;

pub use nuketype::execution::stage_receiver;

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
  pub data:    Box<Data+Send+Share>
}

impl Alien {
  /// Construct an Alien around a given `Routine`.
  pub fn new(routine: Routine, data: Box<Data+Send+Share>) -> Alien {
    Alien {
      routine: routine,
      data:    data
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
               call_pattern_data as Box<Data+Send+Share>)
  }

  /// Construct a oneshot Alien which calls the given `OneshotRoutine` for only
  /// one realization of the Alien. Any further realizations will yield.
  pub fn new_oneshot(routine: OneshotRoutine) -> Alien {
    let oneshot_data = box OneshotData {
      complete: false,
      routine:  routine
    };

    Alien::new(oneshot_alien_routine,
               oneshot_data as Box<Data+Send+Share>)
  }

  /// Turn the function inside a `NativeReceiver` into an Alien.
  ///
  /// As is standard for receiver procedures in Nucleus, the Alien accepts
  /// objects of the following form, and continues to do so indefinitely:
  ///
  /// 1. Ignored (noughty).
  /// 2. Caller.
  /// 3. Subject.
  /// 4. Message.
  pub fn new_from_native_receiver(receiver: fn (&mut Reactor, Params))
                                  -> Alien {

    Alien::new(native_receiver_alien_routine,
               box NativeReceiverData(receiver))
  }

  /// Boxes up a new Alien with the given `routine` and `data`, and tags it with
  /// `name`.
  ///
  /// See `Alien::new()`.
  pub fn create<T: Tag>(
                name:    T,
                routine: Routine,
                data:    Box<Data+Send+Share>)
                -> ObjectRef {

    ObjectRef::store_with_tag(
      box Alien::new(routine, data),
      Meta::with_receiver(stage_receiver),
      name
    )
  }

  /// Boxes up a new call-pattern Alien, and tags it with `name`.
  ///
  /// See `Alien::new_call_pattern()`.
  pub fn call_pattern<T: Tag>(
                      name:    T,
                      routine: CallPatternRoutine,
                      n_args:  uint)
                      -> ObjectRef {

    ObjectRef::store_with_tag(
      box Alien::new_call_pattern(routine, n_args),
      Meta::with_receiver(stage_receiver),
      name
    )
  }

  /// Boxes up a new oneshot Alien, and tags it with `name`.
  ///
  /// See `Alien::new_oneshot()`.
  pub fn oneshot<T: Tag>(
                 name:    T,
                 routine: OneshotRoutine)
                 -> ObjectRef {

    ObjectRef::store_with_tag(
      box Alien::new_oneshot(routine),
      Meta::with_receiver(stage_receiver),
      name
    )
  }

  /// Boxes up a new Alien based on a native receiver function.
  ///
  /// See `Alien::new_from_native_receiver()`.
  pub fn from_native_receiver(receiver: fn (&mut Reactor, Params))
                              -> ObjectRef {

    ObjectRef::store(
      box Alien::new_from_native_receiver(receiver),
      Meta::with_receiver(stage_receiver)
    )
  }

  /// Calls the Alien's routine with the given `reactor` and `response`.
  ///
  /// # Example
  ///
  ///     match alien_ref.lock().try_cast::<Alien>() {
  ///       Ok(alien) => Alien::realize(alien, &reactor, response),
  ///       Err(_)    => fail!("not an alien!")
  ///     }
  pub fn realize<'a, R: Reactor>(
                 alien:    TypedRefGuard<'a, Alien>,
                 reactor:  &mut R,
                 response: ObjectRef) {

    (alien.routine)(alien, reactor, response)
  }
}

impl Nuketype for Alien {
  fn fmt_paws(&self, writer: &mut Writer) -> IoResult<()> {
    write!(writer, "Alien")
  }
}

impl Clone for Alien {
  fn clone(&self) -> Alien {
    Alien {
      routine: self.routine,
      data:    self.data.clone_to_data()
    }
  }
}

/// Types that are acceptable as data for an Alien.
///
/// Used to get around the restriction that `clone()` can't be called on a trait
/// object, to allow `Alien` to still be cloneable. This is a huge hack.
pub trait Data: Any {
  /// Clones and boxes into a `Data` trait object.
  fn clone_to_data(&self) -> Box<Data+Send+Share>;

  /// Gets this Data as an Any reference. Not generally necessary as `AnyRefExt`
  /// is implemented, but it is necessary in order to implement `AnyRefExt` in
  /// the first place.
  fn as_any<'a>(&'a self) -> &'a Any {
    self as &Any
  }

  /// Gets this Data as a mutable Any reference. Not generally necessary as
  /// `AnyMutRefExt` is implemented, but it is necessary in order to implement
  /// `AnyMutRefExt` in the first place.
  fn as_any_mut<'a>(&'a mut self) -> &'a mut Any {
    self as &mut Any
  }
}

impl<T: 'static+Clone+Send+Share> Data for T {
  fn clone_to_data(&self) -> Box<Data+Send+Share> {
    box self.clone() as Box<Data+Send+Share>
  }
}

impl<'a> AnyRefExt<'a> for &'a Data {
  fn is<T:'static>(self) -> bool {
    self.as_any().is::<T>()
  }

  fn downcast_ref<T:'static>(self) -> Option<&'a T> {
    self.as_any().downcast_ref::<T>()
  }
}

impl<'a> AnyMutRefExt<'a> for &'a mut Data {
  fn downcast_mut<T:'static>(self) -> Option<&'a mut T> {
    self.as_any_mut().downcast_mut::<T>()
  }
}

/// A function that implements the logic behind an Alien.
pub type Routine = fn <'a>(
                       alien:    TypedRefGuard<'a, Alien>,
                       reactor:  &mut Reactor,
                       response: ObjectRef);

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
///     alien[] hello world
///
/// Assume this returns `hi`. Timeline:
///
///     alien <- []    ... {caller = []}        ... caller <- alien
///     alien <- hello ... {args   push(hello)} ... caller <- alien
///     alien <- world ... {args   push(world)} ...
///
///       call_pattern_routine(reactor, caller = [], args = [hello, world])
///         -> reactor.stage(caller, hi) ...
///
///     caller <- hi
pub type CallPatternRoutine = fn (reactor: &mut Reactor,
                                  caller:  ObjectRef,
                                  args:    &[ObjectRef]);

/// Internal state for call pattern wrapper.
struct CallPatternData {
  caller:    Option<ObjectRef>,
  args:      Vec<ObjectRef>,
  complete:  bool,
  remaining: uint,
  routine:   CallPatternRoutine
}

impl Clone for CallPatternData {
  fn clone(&self) -> CallPatternData {
    CallPatternData {
      caller:    self.caller.clone(),
      args:      self.args.clone(),
      complete:  self.complete,
      remaining: self.remaining,
      routine:   self.routine
    }
  }
}

/// Function that performs call pattern wrapper.
fn call_pattern_alien_routine<'a>(
                              mut alien: TypedRefGuard<'a, Alien>,
                              reactor:   &mut Reactor,
                              response:  ObjectRef) {

  let (caller, routine, args) = {
    // Do everything we need to do to data in here, so we can drop alien.
    let data = alien.data.downcast_mut::<CallPatternData>().unwrap();

    // Don't do anything if we're complete.
    if data.complete { return }

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
      let alien = alien.unlock();

      debug!("call_pattern_alien_routine: calling {} from {} with {}",
             alien, caller, args.as_slice());

      routine(reactor, caller, args.as_slice())
    },
    None =>
      // Need more args!
      reactor.stage(caller, alien.unlock().clone())
  }
}

/// A function that implements a "oneshot" style Alien.
///
/// This type of alien only ever accepts one argument, after which it is
/// considered to have 'completed' and the routine will never be called again;
/// any further realizations of the Alien result in no response.
pub type OneshotRoutine = fn (reactor:  &mut Reactor,
                              response: ObjectRef);

/// Internal state for oneshot wrapper.
struct OneshotData {
  complete: bool,
  routine:  OneshotRoutine
}

impl Clone for OneshotData {
  fn clone(&self) -> OneshotData {
    OneshotData {
      complete: self.complete,
      routine:  self.routine
    }
  }
}

/// Function that performs oneshot wrapper.
fn oneshot_alien_routine<'a>(
                          mut alien: TypedRefGuard<'a, Alien>,
                          reactor:   &mut Reactor,
                          response:  ObjectRef) {

  let routine = {
    let data = alien.data.downcast_mut::<OneshotData>().unwrap();

    if data.complete {
      return;
    } else {
      data.complete = true;
    }

    data.routine
  };

  drop(alien);

  routine(reactor, response)
}

/// Internal state for native receiver wrapper.
struct NativeReceiverData(fn (&mut Reactor, Params));

impl Clone for NativeReceiverData {
  fn clone(&self) -> NativeReceiverData {
    let NativeReceiverData(receiver) = *self;
    NativeReceiverData(receiver)
  }
}

/// Function that performs native receiver wrapper.
fn native_receiver_alien_routine<'a>(
                                 mut alien: TypedRefGuard<'a, Alien>,
                                 reactor:   &mut Reactor,
                                 response:  ObjectRef) {

  let NativeReceiverData(receiver) =
    *alien.data.downcast_mut::<NativeReceiverData>().unwrap();

  drop(alien);

  let params = {
    let params_obj = response.lock();
    let members    = &params_obj.meta().members;

    match (members.get(1), members.get(2), members.get(3)) {
      (Some(caller), Some(subject), Some(message)) =>
        Params {
          caller:  caller.to().clone(),
          subject: subject.to().clone(),
          message: message.to().clone()
        },

      _ => {
        // Malformed. Warn and unstage.
        warn!(concat!("native_receiver_alien_routine received",
                      " malformed params object {}"),
              response);

        return
      }
    }
  };

  receiver(reactor, params)
}
