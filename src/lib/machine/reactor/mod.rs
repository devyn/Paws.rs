//! Reactors are the evaluation cores of Paws.
//!
//! This module contains several different types of reactors, suitable for
//! different purposes, including a `MockReactor` intended for testing.

use machine::Machine;

use object::{Object, ObjectRef, ObjectRefGuard};
use object::{ObjectReceiver, NativeReceiver};
use object::Params;

use object::thing::Thing;
use object::execution::Execution;
use object::alien::Alien;

use util::clone;

pub use self::mock::MockReactor;
pub use self::serial::SerialReactor;
pub use self::parallel::{ReactorPool, ParallelReactor};

mod mock;
mod serial;
mod parallel;

#[cfg(test)]
mod tests;

/// A single Paws reactor.
///
/// Responsible for a single Machine's Unit. In the future, Machines will be
/// split so that they can have multiple Units and also talk to other Machines,
/// possibly on a network.
///
/// May be part of a pool, in which the reactors are expected to communicate
/// with each other transparently.
pub trait Reactor {
  /// Stages an execution for reaction with a response.
  ///
  /// The reactor that handles the reaction may not necessarily be this same
  /// reactor: the reactor may spill its work onto another reactor in its pool.
  fn stage(&mut self, execution: ObjectRef, response: ObjectRef);

  /// Adds a stall handler, which will be called the next time the reactor finds
  /// itself unable to progress further (i.e., no work and no pending external
  /// actions).
  ///
  /// If the reactor is part of a pool, the handler will only be called if the
  /// entire pool runs out of work.
  fn on_stall(&mut self, handler: proc (&mut Reactor));

  /// Immediately terminates the reactor.
  ///
  /// If the reactor is part of a pool, the other reactors will be terminated as
  /// well.
  fn stop(&mut self);

  /// Gets a reference to the machine this reactor is associated with.
  fn machine(&self) -> &Machine;
}

/// Describes a Combination of a `message` against a `subject`.
#[deriving(Clone, PartialEq, Eq, Show)]
pub struct Combination {
  /// The left hand side, what the `message` is combined *against*.
  ///
  /// If `None`, the Combination shall be against the calling Execution's
  /// locals.
  pub subject: Option<ObjectRef>,

  /// The right hand side, what the `subject` is combined *with*.
  pub message: ObjectRef
}

/// Implements the combination algorithm, finding the appropriate receiver and
/// then invoking it.
///
/// If the `combination`'s `subject` is `None`, it will be interpreted to be
/// the `caller`'s "locals".
///
/// From the spec:
///
/// > **Finding the receiver for a given Object**
/// >
/// > A 'receiver' is an Execution associated with a given object, one
/// responsible for handling combinations when that object is the `subject` of
/// the combination.
/// >
/// > 1. If the `subject` has no `receiver` property set, then an Execution
/// >    implementing the "default receiver" algorithm for that type of object
/// >    is the result of this algorithm. (Each type described above includes
/// >    a description of that type's default receiver's algorithm.)
/// >
/// > 2. If the `subject` has a `receiver` property, and the value of that
/// >    property is queueable (i.e. an Execution), then that Execution is the
/// >    result of this algorithm.
/// >
/// > 3. If the `subject`'s `receiver` is not queueable (that is, not an
/// >    Execution), then recursively apply this algorithm starting at 1, with
/// >    that `receiver` as the `subject` *for the purposes of this
/// >    algorithm*. (*Not* for the consumer of this algorithm, who will have
/// >    their own reference to the original `subject`.)
/// >
/// > **Rationale:** The recursive nature of this process allows object-system
/// > designers to wrap their receiver(s) in metadata, or otherwise abstract
/// > them away.
pub fn combine<'a, R: Reactor>(
               reactor:     &mut R,
               caller:      ObjectRefGuard<'a>,
               combination: Combination) {

  // Get the actual subject and message, interpreting a None subject in the
  // combination provided as "locals".
  let (subject, message) = match combination {
    Combination { subject: Some(subject),
                  message: message } =>
      (subject, message),

    Combination { subject: None,
                  message: message } => {

      let members = &caller.deref().meta().members;

      // Find the caller's locals and make that the subject.
      //
      // If we can't find the locals, immediately give up -- we can't continue,
      // since the Execution is obviously totally fucked up.
      match members.lookup_pair(&reactor.machine().locals_sym) {
        Some(locals) => (locals, message),
        None         => fail!("Execution is missing locals!")
      }
    }
  };

  // We no longer need to look at any of the caller's properties.
  let caller = caller.unlock().clone();

  // Perform the receiver-finding algorithm, using `use_receiver_of` to
  // iterate through until we find the receiver we want to use.
  let mut use_receiver_of = subject.clone();
  loop {
    // We have to clone this again because rustc apparently isn't smart enough
    // to realize that `drop(current_target)` means that use_receiver_of is no
    // longer borrowed >_>
    let current_target_ref = use_receiver_of.clone();
    let current_target     = current_target_ref.lock();

    match current_target.deref().meta().receiver.clone() {
      // If the receiver is a NativeReceiver, then call the function it
      // contains.
      NativeReceiver(function) => {
        drop(current_target); // Release the lock ASAP.

        return function(reactor, Params {
          caller:  caller,
          subject: subject,
          message: message
        })
      },

      // Otherwise, we need to check if this receiver is queueable (Execution
      // or Alien) or not.
      ObjectReceiver(receiver) => {
        drop(current_target); // Release the lock ASAP.

        match clone::queueable(&receiver, reactor.machine()) {
          Some(clone) => {
            // If it is, we construct a params object `[, caller, subject,
            // message]` and `React` a clone of the receiver with the params
            // object as the response.
            //
            // TODO: Find a way to not have to clone it all the time.
            let mut params = box Thing::new();

            params.meta_mut().members.set(1, caller);
            params.meta_mut().members.set(2, subject);
            params.meta_mut().members.set(3, message);

            return reactor.stage(clone, ObjectRef::new(params))
          },

          None => {
            // If it isn't, we need to loop through this whole thing again, with
            // this receiver as `use_receiver_of`.
            use_receiver_of = receiver;
          }
        }
      }
    }
  }
}

/// Realizes an Execution (or Alien) with the given response.
///
/// In the case of Executions, this causes the Execution to be advanced with
/// the response and the resulting Combination to be evaluated.
///
/// Aliens are simply `Alien::realize()`d with the response, invoking their
/// routine.
pub fn realize<R: Reactor>(
               reactor:       &mut R,
               execution_ref: ObjectRef,
               response_ref:  ObjectRef) {
  // Detect whether `execution_ref` is an Execution, an Alien, or
  // something else, and handle those cases separately.
  match execution_ref.lock().try_cast::<Execution>() {
    Ok(mut execution) => {
      // For an Execution, we just want to advance() it and have the
      // Machine process the combination if there was one.

      debug!("realize execution {} \t<-- {}",
        execution_ref, response_ref);

      match execution.advance(&execution_ref, response_ref) {
        Some(combination) =>
          // Calls the receiver and all that jazz.
          combine(reactor, execution.into_untyped(), combination),

        None =>
          // This execution is already complete, so we can't do anything.
          debug!("execution {} complete", execution_ref)
      }
    },

    Err(execution_ish) =>
      match execution_ish.try_cast::<Alien>() {
        Ok(alien) => {
          // Aliens are a bit different. They handle unlocking themselves
          // at a point which they see fit, so we give them the lock.

          debug!("realize alien     {} \t<-- {}",
            execution_ref, response_ref);

          Alien::realize(alien, reactor, response_ref)
        },

        Err(_) =>
          // Finally, if it was neither an Execution nor an Alien, it
          // really shouldn't have been given to us and we'll just pretend it
          // wasn't.
          warn!("tried to realize non-queueable {}!", execution_ref)
      }
  }
}
