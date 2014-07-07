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

use object::*;
use object::execution::Execution;
use object::alien::Alien;
use object::thing::Thing;

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
pub fn queueable(from: &ObjectRef) -> Option<ObjectRef> {
  // XXX: This currently clones the receiver too... should it not?

  match from.lock().try_cast::<Execution>() {

    Ok(execution) =>
      Some(ObjectRef::new_clone_of(from, box execution.deref().clone())),

    Err(unknown) => match unknown.try_cast::<Alien>() {

      Ok(alien) =>
        Some(ObjectRef::new_clone_of(from, box alien.deref().clone())),

      Err(_) =>
        None
    }
  }
}
