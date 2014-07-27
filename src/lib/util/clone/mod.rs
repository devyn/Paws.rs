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

use object::{ObjectRef, Meta};
use nuketype::{Thing, Execution, Alien, Locals};

/// Creates a new Thing object from the metadata of the given object.
pub fn to_thing(from: &ObjectRef) -> ObjectRef {
  // TODO: Ask @ELLIOTTCABLE if this is supposed to copy the receiver too
  // (Paws.js says no)

  let mut meta = Meta::new();

  meta.members = from.lock().meta().members.clone();

  Thing::create(meta)
}

/// Correctly clones an `Execution` *or* `Alien`.
///
/// Useful because in Paws-world, they're supposed to behave the same way.
///
/// Returns `None` for non-stageables.
pub fn stageable(from:       &ObjectRef,
                 locals_sym: &ObjectRef)
                 -> Option<ObjectRef> {

  stageable_with_details(from, locals_sym).map(|result| result.stageable)
}

/// The result of `stageable_with_details()`.
pub struct StageableWithDetailsResult {
  /// The cloned object.
  pub stageable:        ObjectRef,

  /// The cloned locals and its metadata version.
  ///
  /// Only present if an Execution was cloned (aliens don't have `locals`).
  pub locals:           Option<(ObjectRef, uint)>,

  /// The nuketype version of the object that was cloned.
  pub nuketype_version: uint,

  /// The metadata version of the object that was cloned.
  pub meta_version:     uint
}

/// Clones an `Execution` or `Alien` with details about the versions of
/// relevant parts at the time of cloning.
pub fn stageable_with_details(from:       &ObjectRef,
                              locals_sym: &ObjectRef)
                              -> Option<StageableWithDetailsResult> {

  // XXX: This currently clones the receiver too... should it not?
  //
  // We lock it first to ensure that the versions are correct. (Version can only
  // be changed while locked.)
  let unknown = from.lock();

  let nuketype_version = from.nuketype_version();
  let meta_version     = from.meta_version();

  match unknown.try_cast::<Execution>() {

    Ok(execution) => {
      let     new_execution = box execution.deref().clone();
      let mut new_meta      = execution.meta().clone();

      execution.unlock();

      let locals_version;

      let new_locals = {

        let locals_ref = new_meta.members
                           .lookup_pair(locals_sym)
                           .expect("Execution is missing locals!");

        let locals = locals_ref.lock().try_cast::<Locals>()
                       .ok().expect("locals should be a Locals!");

        locals_version = locals_ref.meta_version();

        ObjectRef::store_with_tag(
          box locals.deref().clone(),
          locals.meta().clone(),
          locals_ref.tag())
      };

      new_meta.members
        .push_pair_to_child(locals_sym.clone(), new_locals.clone());

      Some(StageableWithDetailsResult {
        stageable:        ObjectRef::store_with_tag(
                            new_execution, new_meta, from.tag()),
        locals:           Some((new_locals, locals_version)),
        nuketype_version: nuketype_version,
        meta_version:     meta_version
      })
    },

    Err(unknown) => match unknown.try_cast::<Alien>() {

      Ok(alien) => {
        Some(StageableWithDetailsResult {
          stageable:        ObjectRef::store_with_tag(
                              box alien.deref().clone(),
                              alien.meta().clone(),
                              from.tag()),
          locals:           None,
          nuketype_version: nuketype_version,
          meta_version:     meta_version
        })
      },

      Err(_) =>
        None
    }
  }
}
