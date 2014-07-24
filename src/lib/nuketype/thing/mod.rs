//! Things contain Object metadata and nothing more.

use object::{ObjectRef, Meta, Tag};

use nuketype::Nuketype;

use std::io::IoResult;

/// A generic Nuketype that contains no data. An object that boxes this type is
/// only useful for its metadata.
///
/// # Example
///
/// Without convenience methods:
///
///     ObjectRef::store(box Thing, Meta::new())
///
/// With convenience methods:
///
///     Thing::empty()
#[deriving(Clone)]
pub struct Thing;

impl Thing {
  /// Boxes a new Thing with the given Meta.
  ///
  /// # Example
  ///
  /// Constructing a new Thing by cloning the Meta of an existing object:
  ///
  ///    Thing::create(object.lock().meta().clone())
  pub fn create(meta: Meta) -> ObjectRef {
    ObjectRef::store(box Thing, meta)
  }

  /// Boxes a new Thing with the given Meta, and a tag.
  pub fn tagged<T: Tag>(meta: Meta, tag: T) -> ObjectRef {
    ObjectRef::store_with_tag(box Thing, meta, tag)
  }

  /// Boxes a new Thing with empty Meta (`Meta::new()`).
  pub fn empty() -> ObjectRef {
    Thing::create(Meta::new())
  }

  /// Boxes a new Thing with metadata in the form of a Nucleus-lookup-style
  /// pair.
  ///
  /// The resulting members structure looks like this:
  ///
  /// 1. A hole (`None`).
  /// 2. Non-child: `key`.
  /// 3. Non-child: `value`.
  pub fn pair(key: ObjectRef, value: ObjectRef) -> ObjectRef {
    let mut meta = Meta::new();

    meta.members.set(1, key);
    meta.members.set(2, value);

    ObjectRef::store(box Thing, meta)
  }

  /// Boxes a new Thing with metadata in the form of a Nucleus-lookup-style
  /// pair, where the value is marked as a child relationship.
  ///
  /// The resulting members structure looks like this:
  ///
  /// 1. A hole (`None`).
  /// 2. Non-child: `key`.
  /// 3. Child: `value`.
  pub fn pair_to_child(key: ObjectRef, value: ObjectRef) -> ObjectRef {
    let mut meta = Meta::new();

    meta.members.set(1, key);
    meta.members.set_child(2, value);

    ObjectRef::store(box Thing, meta)
  }
}

impl Nuketype for Thing {
  fn fmt_paws(&self, writer: &mut Writer) -> IoResult<()> {
    write!(writer, "Thing")
  }
}
