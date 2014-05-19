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
  fn as_mut_any<'a>(&'a mut self) -> &'a mut Any {
    self as &mut Any
  }
}

/// A reference to an object.
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

impl Clone for ObjectRef {
  fn clone(&self) -> ObjectRef {
    ObjectRef { reference: self.reference.clone() }
  }
}
