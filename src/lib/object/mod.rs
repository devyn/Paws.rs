//! Paws objects, and a trait that they all share

use std::any::*;
use sync::{Arc, Mutex};
use std::io::IoResult;
use machine::Machine;

pub mod empty;
pub mod symbol;
pub mod execution;
pub mod alien;

#[cfg(test)]
mod tests;

/// The interface that all Paws Objects must implement.
pub trait Object {
  /// Formats a Paws Object for debugging purposes.
  ///
  /// **TODO:** `machine` should really be moved out of here if possible. It's
  /// only required for looking symbols up in the machine's `SymbolMap`.
  fn fmt_paws(&self, writer: &mut Writer, machine: &Machine) -> IoResult<()>;

  /// Converts an Object trait object to an Any trait object.
  ///
  /// This is useful for attempting to convert an Object trait object into its
  /// original type, for example, getting the Symbol within an Object, via
  /// `as_ref()` on the resulting `&Any`.
  ///
  /// # Example
  ///
  ///     let maybe_symbol: Option<&Symbol> = object.as_any().as_ref();
  ///     match maybe_symbol {
  ///       Some(symbol) => println!("{}", symbol.name(&machine.symbol_map)),
  ///       None         => fail!("expected Symbol")
  ///     }
  fn as_any<'a>(&'a self) -> &'a Any {
    self as &Any
  }

  /// Same as `as_any()` but for a mutable ref.
  fn as_any_mut<'a>(&'a mut self) -> &'a mut Any {
    self as &mut Any
  }

  /// Get access to the Object's metadata, including members and such.
  fn meta<'a>(&'a self) -> &'a Meta;

  /// Get mutable access to the Object's metadata.
  fn meta_mut<'a>(&'a mut self) -> &'a mut Meta;

  /// Implements the 'default receiver' of an Object.
  ///
  /// Called by the Reactor if the Object does not have a different 'receiver'
  /// explicitly set within its metadata.
  ///
  /// The default implementation of this function resumes the caller with the
  /// result of interpreting the Object's members as key-value pairs and
  /// grabbing the value associated with the message as the key. If the key
  /// could not be found, the caller is **not** resumed.
  ///
  /// This implementation is probably suitable for most non-execution-like
  /// Object. Execution-like Objects obviously will want to override this
  /// function.
  fn combine(&mut self, machine: &mut Machine, caller: ObjectRef,
             message: ObjectRef) {
    match self.meta().lookup_member(message) {
      Some(object_ref) =>
        machine.stage(caller, object_ref, None),
      None => ()
    }
  }
}

/// A reference to an object. Thread-safe.
///
/// Prefer immutable access (`read()`) unless necessary. Multiple tasks can read
/// in parallel, but only one may write at a time.
#[deriving(Clone)]
pub struct ObjectRef {
  priv reference: Arc<Mutex<~Object:Send+Share>>
}

impl ObjectRef {
  /// Boxes an Object trait into an Object reference.
  pub fn new(object: ~Object:Send+Share) -> ObjectRef {
    ObjectRef { reference: Arc::new(Mutex::new(object)) }
  }
}

impl Eq for ObjectRef {
  fn eq(&self, other: &ObjectRef) -> bool {
    (&*self.reference  as *Mutex<~Object:Send+Share>) ==
    (&*other.reference as *Mutex<~Object:Send+Share>)
  }
}

impl TotalEq for ObjectRef { }

impl Deref<Mutex<~Object:Send+Share>> for ObjectRef {
  fn deref<'a>(&'a self) -> &'a Mutex<~Object:Send+Share> {
    &*self.reference
  }
}

/// A link to an object, to be referenced within an object's 'members' list.
#[deriving(Clone)]
pub struct Relationship {
  priv to:       ObjectRef,
  priv is_child: bool
}

impl Relationship {
  /// Creates a new non-child relationship.
  pub fn new(to: ObjectRef) -> Relationship {
    Relationship { to: to, is_child: false }
  }

  /// Creates a new child relationship. See `is_child`.
  pub fn new_child(to: ObjectRef) -> Relationship {
    Relationship { to: to, is_child: true }
  }

  /// Indicates whether the link is a 'child relationship', i.e. an owned
  /// reference. When an execution requests 'responsibility' over a given
  /// object, it must also implicitly acquire responsibility over all of that
  /// object's child relationships recursively (but not non-child
  /// relationships).
  pub fn is_child(&self) -> bool {
    self.is_child
  }

  /// The object this relationship points to.
  pub fn to<'a>(&'a self) -> &'a ObjectRef {
    &self.to
  }
}

/// Object metadata -- this is universal for all objects, and required in order
/// to implement the `Object` trait.
#[deriving(Clone)]
pub struct Meta {
  /// A list of Relationships that make up the Object's members.
  ///
  /// The vector is of `Option<Relationship>` to allow for holes -- when a
  /// member is inserted at a position beyond the size of the vector, the gap is
  /// filled with `None`s that will act as if the element does not exist.
  ///
  /// Note that 'nuclear' algorithms (i.e. those part of Paws' Nucleus, which is
  /// what Paws.rs strives to implement) should never assume anything about the
  /// first element of the list and should instead start from the second element
  /// unless specifically requested not to, as per the 'noughty' rule (see
  /// spec).
  pub members: Vec<Option<Relationship>>
}

impl Meta {
  /// Helpful constructor with some sensible default values.
  ///
  /// * `members`: empty vec
  pub fn new() -> Meta {
    Meta {
      members: Vec::new()
    }
  }

  /// Searches for a given key within `members` according to Paws' "nuclear"
  /// association-list semantics.
  ///
  /// # Example
  ///
  /// Using JavaScript-like syntax to represent members, ignoring other
  /// properties of the objects:
  ///
  ///     [, [, hello, world], [, foo, bar], [, hello, goodbye]]
  ///
  /// When looking up `hello`:
  ///
  /// * Iteration is done in reverse order; key and value are second and
  ///   third elements respectively, so result is `Some(goodbye)`
  fn lookup_member(&self, key: ObjectRef) -> Option<ObjectRef> {
    for maybe_relationship in self.members.tail().iter().rev() {
      match maybe_relationship {
        &Some(ref relationship) => {
          let object  = relationship.to().lock();
          let members = &object.deref().meta().members;

          if members.len() >= 3 {
            match (members.get(1), members.get(2)) {
              (&Some(ref rel_key), &Some(ref rel_value)) =>
                if rel_key.to() == &key {
                  return Some(rel_value.to().clone())
                },
              _ => ()
            }
          }
        },
        _ => ()
      }
    }
    None
  }
}
