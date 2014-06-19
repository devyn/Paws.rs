//! Paws objects, and a trait that they all share

use std::any::*;
use sync::{Arc, RWLock};
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
}

/// A reference to an object. Thread-safe.
///
/// Prefer immutable access (`read()`) unless necessary. Multiple tasks can read
/// in parallel, but only one may write at a time.
#[deriving(Clone)]
pub struct ObjectRef {
  priv reference: Arc<RWLock<~Object:Send+Share>>
}

impl ObjectRef {
  /// Boxes an Object trait into an Object reference.
  pub fn new(object: ~Object:Send+Share) -> ObjectRef {
    ObjectRef { reference: Arc::new(RWLock::new(object)) }
  }
}

impl Eq for ObjectRef {
  fn eq(&self, other: &ObjectRef) -> bool {
    (&*self.reference  as *RWLock<~Object:Send+Share>) ==
    (&*other.reference as *RWLock<~Object:Send+Share>)
  }
}

impl TotalEq for ObjectRef { }

impl Deref<RWLock<~Object:Send+Share>> for ObjectRef {
  fn deref<'a>(&'a self) -> &'a RWLock<~Object:Send+Share> {
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
}

impl Deref<ObjectRef> for Relationship {
  fn deref<'a>(&'a self) -> &'a ObjectRef {
    &self.to
  }
}

/// Object metadata -- this is universal for all objects, and required in order
/// to implement the `Object` trait.
#[deriving(Clone)]
pub struct Meta {
  /// A list of Relationships that make up the Object's members.
  ///
  /// Note that 'nuclear' algorithms (i.e. those part of Paws' Nucleus, which is
  /// what Paws.rs strives to implement) should never assume anything about the
  /// first element of the list and should instead start from the second element
  /// unless specifically requested not to, as per the 'noughty' rule (see
  /// spec).
  pub members: Vec<Relationship>
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
}
