//! Duck-typing utilities for cloning Objects from and to certain categories.
//!
//! The names make more sense if you only import the module itself:
//!
//!     use util::clone;
//!
//!     //...
//!
//!     clone::to_thing(...);
//!     clone::queueable(...);

use machine::Machine;

use object::*;
use object::execution::Execution;
use object::alien::Alien;
use object::thing::Thing;
use object::locals::Locals;

/// Creates a new Thing from the metadata of the given object.
pub fn to_thing(from: &ObjectRef) -> Thing {
  // TODO: Ask @ELLIOTTCABLE if this is supposed to copy the receiver too
  // (Paws.js says no)

  let mut meta = Meta::new();

  meta.members = from.lock().meta().members.clone();

  Thing::from_meta(meta)
}

/// Correctly clones `Execution`s *or* `Alien`s.
///
/// Useful because in Paws-world, they're supposed to behave the same way.
pub fn queueable(from: &ObjectRef, machine: &Machine) -> Option<ObjectRef> {
  // XXX: This currently clones the receiver too... should it not?

  match from.lock().try_cast::<Execution>() {

    Ok(execution) => {
      let mut new_execution = box execution.deref().clone();

      let locals = execution.deref().meta().members
                     .lookup_pair(&machine.locals_sym)
                     .expect("Execution is missing locals!");

      execution.unlock();

      let new_locals = ObjectRef::new_with_tag(
        box locals.lock().try_cast::<Locals>()
              .ok().expect("locals should be a Locals!")
              .clone(),
        locals.tag());

      new_execution.meta_mut().members
        .push_pair_to_child(machine.locals_sym.clone(), new_locals);

      Some(ObjectRef::new_with_tag(new_execution, from.tag()))
    },

    Err(unknown) => match unknown.try_cast::<Alien>() {

      Ok(alien) =>
        Some(ObjectRef::new_with_tag(box alien.deref().clone(), from.tag())),

      Err(_) =>
        None
    }
  }
}
