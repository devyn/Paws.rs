//! Paws objects, and a trait that they all share

use std::any::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::io::IoResult;
use machine::Machine;

pub mod symbol;
pub mod execution;

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

  /// A list of Relationships that make up the Object's members.
  ///
  /// Note that 'nuclear' algorithms (i.e. those part of Paws' Nucleus, which is
  /// what Paws.rs strives to implement) should never assume anything about the
  /// first element of the list and should instead start from the second element
  /// unless specifically requested not to, as per the 'noughty' rule (see
  /// spec).
  fn members<'a>(&'a self) -> &'a Vec<Relationship>;

  /// A mutable reference to the list of Relationships that make up the Object's
  /// members.
  ///
  /// See `members` for more information.
  fn members_mut<'a>(&'a mut self) -> &'a mut Vec<Relationship>;
}

/// A reference to an object.
#[deriving(Clone)]
pub struct ObjectRef {
  reference: Rc<RefCell<~Object:'static>>
}

impl ObjectRef {
  /// Boxes an Object trait into an Object reference.
  pub fn new(object: ~Object:'static) -> ObjectRef {
    ObjectRef { reference: Rc::new(RefCell::new(object)) }
  }
}

impl Eq for ObjectRef {
  fn eq(&self, other: &ObjectRef) -> bool {
    (&*self.reference  as *RefCell<~Object:'static>) ==
    (&*other.reference as *RefCell<~Object:'static>)
  }
}

impl TotalEq for ObjectRef { }

impl Deref<RefCell<~Object:'static>> for ObjectRef {
  fn deref<'a>(&'a self) -> &'a RefCell<~Object:'static> {
    &*self.reference
  }
}

/// A link to an object, to be referenced within an object's 'members' list.
#[deriving(Clone)]
pub struct Relationship {
  to:       ObjectRef,
  is_child: bool
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
