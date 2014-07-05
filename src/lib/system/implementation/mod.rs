//! Implementation-specific aliens.

#![allow(unused_variable)]

use object::*;
use object::thing::Thing;
use object::alien::Alien;

use machine::*;

use util::namespace::*;

use std::any::*;

pub mod console;

#[cfg(test)]
mod tests;

/// Generates an `implementation` namespace object.
pub fn make(machine: &Machine) -> ObjectRef {
  let mut implementation =
    box Thing::from_meta(Meta::with_receiver(namespace_receiver));

  {
    let mut add = NamespaceBuilder::new(machine, &mut *implementation);

    add.namespace(    "console",                 console::make                );
    add.factory(      "void",                    void                         );
    add.call_pattern( "stop",                    stop, 0                      );
  }

  ObjectRef::new(implementation)
}

/// Acts as a void, accepting and discarding objects and then returning itself
/// to the caller indefinitely.
///
/// Often used to do separate things in sequence, starting from `locals` each
/// time.
///
/// # Queueing semantics
///
/// 1. Accept and store the caller.
/// 2. Accept, discard, and then queue caller for realization with self.
/// 3. Repeat step 2.
///
/// # Example
///
///     implementation void[] a b c [foo] [bar baz]
pub fn void(machine: &Machine) -> Alien {
  #[deriving(Clone)]
  struct VoidCaller(Option<ObjectRef>);

  fn void_routine<'a>(
                  mut alien: TypedRefGuard<'a, Alien>,
                  machine:   &Machine,
                  response:  ObjectRef)
                  -> Reaction {

    let caller: ObjectRef;

    match alien.data.as_mut::<VoidCaller>() {
      Some(&VoidCaller(Some(ref stored_caller))) => {
        // We've already got the caller and we need to use it.
        caller = stored_caller.clone();
      },

      Some(&VoidCaller(ref mut stored_caller)) => {
        // This response is the caller; store it.
        *stored_caller = Some(response.clone());
        caller = response;
      },

      None =>
        fail!("void_routine called on a non-void() Alien!")
    }

    React(caller, alien.unlock().clone())
  }

  Alien::new(void_routine, box VoidCaller(None))
}

/// Halts the machine by terminating its queue.
///
/// # Call pattern arguments
///
/// No arguments.
///
/// # Example
///
///     implementation stop[]
pub fn stop(machine: &Machine,
            caller:  ObjectRef,
            args:    &[ObjectRef])
            -> Reaction {
  machine.stop();
  Yield
}
