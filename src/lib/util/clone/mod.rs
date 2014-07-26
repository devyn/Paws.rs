//! Duck-typing utilities for cloning Objects from and to certain categories.
//!
//! The names make more sense if you only import the module itself:
//!
//!     use util::clone;
//!
//!     //...
//!
//!     clone::to_thing(...);
//!     clone::stageable(...);

use object::{ObjectRef, Meta, Tag};
use nuketype::{Thing, Execution, Alien, Locals};
use machine::Machine;

use std::sync::Arc;

/// Creates a new Thing object from the metadata of the given object.
pub fn to_thing(from: &ObjectRef) -> ObjectRef {
  // TODO: Ask @ELLIOTTCABLE if this is supposed to copy the receiver too
  // (Paws.js says no)

  let mut meta = Meta::new();

  meta.members = from.lock().meta().members.clone();

  Thing::create(meta)
}

/// Correctly clones `Execution`s *or* `Alien`s.
///
/// Useful because in Paws-world, they're supposed to behave the same way.
pub fn stageable(from: &ObjectRef, machine: &Machine) -> Option<ObjectRef> {
  // XXX: This currently clones the receiver too... should it not?

  match from.lock().try_cast::<Execution>() {

    Ok(execution) => {
      let     new_execution = box execution.deref().clone();
      let mut new_meta      = execution.meta().clone();

      execution.unlock();

      let new_locals = {

        let locals_ref = new_meta.members
                           .lookup_pair(&machine.locals_sym)
                           .expect("Execution is missing locals!");

        let locals = locals_ref.lock().try_cast::<Locals>()
                       .ok().expect("locals should be a Locals!");

        ObjectRef::store_with_tag(
          box locals.deref().clone(),
          locals.meta().clone(),
          locals_ref.tag())
      };

      new_meta.members
        .push_pair_to_child(machine.locals_sym.clone(), new_locals);

      Some(ObjectRef::store_with_tag(
             new_execution, new_meta, from.tag()))
    },

    Err(unknown) => match unknown.try_cast::<Alien>() {

      Ok(alien) => {
        let tag: Option<&Arc<String>> = from.tag();
        debug!("clone::stageable: {}", tag.to_tag().map(|s|(*s).clone()));
        Some(ObjectRef::store_with_tag(
               box alien.deref().clone(), alien.meta().clone(), tag))
      },

      Err(_) =>
        None
    }
  }
}
