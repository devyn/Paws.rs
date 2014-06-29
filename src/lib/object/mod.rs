//! Paws objects, and a trait that they all share

use std::any::*;
use sync::{Arc, Mutex, MutexGuard};
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
  fn fmt_paws(&self, writer: &mut Writer) -> IoResult<()>;

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

  /// Returns a NativeReceiver that implements the 'default receiver' of an
  /// Object type. The `self` reference given should be ignored; it is purely
  /// for typing through a trait object. Additionally, an Object implementation
  /// should probably have another way to access this receiver function.
  ///
  /// The default implementation is provided by `lookup_receiver`.
  ///
  /// See the spec for rationale.
  fn default_receiver(&self) -> NativeReceiver {
    lookup_receiver
  }
}

/// A receiver that simply calls `lookup_member()` on the subject's Meta with
/// the message as its argument.
///
/// If the lookup succeeds, the caller is staged with the result as the
/// response. If the lookup does not succeed, the caller is not re-staged.
///
/// This receiver is the default receiver for all Object types, unless
/// overridden.
#[allow(unused_variable)]
pub fn lookup_receiver(machine: &Machine, params: Params) -> Reaction {
  let subject = params.subject.lock();

  match subject.deref().meta()
               .lookup_member(&params.message) {
    Some(value) =>
      React(params.caller.clone(), value),
    None =>
      Yield
  }
}

/// A reference to an object. Use `lock()` to gain access to the `Object`
/// underneath.
#[deriving(Clone)]
pub struct ObjectRef {
  priv reference:  Arc<Mutex<~Object:Send+Share>>,
  priv symbol_ref: Option<Arc<~str>>
}

impl ObjectRef {
  /// Boxes an Object trait into an Object reference.
  pub fn new(object: ~Object:Send+Share) -> ObjectRef {
    ObjectRef {
      reference:  Arc::new(Mutex::new(object)),
      symbol_ref: None
    }
  }

  /// Boxes a Symbol into a Symbol reference.
  ///
  /// This is a special case to allow for lockless symbol comparison
  /// (`ObjectRef::eq_as_symbol()`). All Symbol-containing ObjectRefs are
  /// assumed to have been created this way; behavior is undefined if they are
  /// created with `ObjectRef::new()` instead.
  pub fn new_symbol(symbol: ~symbol::Symbol) -> ObjectRef {
    ObjectRef {
      symbol_ref: Some(symbol.name_ptr()),
      reference:  Arc::new(Mutex::new(symbol as ~Object:Send+Share))
    }
  }

  /// Obtain exclusive access to the Object this reference points to.
  ///
  /// The Object can be accessed via the returned RAII guard. The returned guard
  /// also contains a reference to this ObjectRef.
  pub fn lock<'a>(&'a self) -> ObjectRefGuard<'a> {
    ObjectRefGuard {
      object_ref: self,
      guard:      self.reference.lock()
    }
  }

  /// Returns true if both `ObjectRef`s are Symbol references that point at the
  /// same Symbol string.
  pub fn eq_as_symbol(&self, other: &ObjectRef) -> bool {
    match (&self.symbol_ref, &other.symbol_ref) {
      (&Some(ref a), &Some(ref b)) =>
        (&**a as *~str) == (&**b as *~str),

      _ => false
    }
  }

  /// If this `ObjectRef` is a Symbol reference, returns a reference to the
  /// pointer to the Symbol's name.
  pub fn symbol_ref<'a>(&'a self) -> Option<&'a Arc<~str>> {
    self.symbol_ref.as_ref()
  }
}

impl Eq for ObjectRef {
  fn eq(&self, other: &ObjectRef) -> bool {
    (&*self.reference  as *Mutex<~Object:Send+Share>) ==
    (&*other.reference as *Mutex<~Object:Send+Share>)
  }
}

impl TotalEq for ObjectRef { }

/// Represents exclusive access to an object.
///
/// Exclusive access is dropped when this guard is dropped.
pub struct ObjectRefGuard<'a> {
  priv object_ref: &'a ObjectRef,
  priv guard:      MutexGuard<'a, ~Object:Send+Share>
}

impl<'a> ObjectRefGuard<'a> {
  /// Unlocks the guard, returning a reference to the ObjectRef that was used to
  /// create the guard originally.
  ///
  /// The guard is consumed in an attempt to prevent unintentional deadlocks due
  /// to double-locking within the task.
  pub fn unlock(self) -> &'a ObjectRef {
    self.object_ref
  }

  /// Compares the internal ObjectRef to another reference without unlocking.
  ///
  /// This can be used to check whether you already possess the guard to a given
  /// ObjectRef.
  pub fn ref_eq(&self, other: &ObjectRef) -> bool {
    self.object_ref == other
  }

  /// Attempts to convert an untyped `ObjectRefGuard` to a `TypedRefGuard` of a
  /// specific type.
  ///
  /// Consumes this guard and wraps into a `TypedRefGuard` if the type of the
  /// Object within matched the requested type, otherwise, returns this guard
  /// to be used again.
  pub fn try_cast<T:'static>(self)
                  -> Result<TypedRefGuard<'a, T>, ObjectRefGuard<'a>> {
    if self.deref().as_any().is::<T>() {
      Ok(TypedRefGuard { object_ref_guard: self })
    } else {
      Err(self)
    }
  }
}

impl<'a> Deref<~Object:Send+Share> for ObjectRefGuard<'a> {
  fn deref<'a>(&'a self) -> &'a ~Object:Send+Share {
    self.guard.deref()
  }
}

impl<'a> DerefMut<~Object:Send+Share> for ObjectRefGuard<'a> {
  fn deref_mut<'a>(&'a mut self) -> &'a mut ~Object:Send+Share {
    self.guard.deref_mut()
  }
}

/// Allows pre-typechecked guards to be marked with their types to remove
/// redundant boilerplate when passing `ObjectRefGuard`s around.
pub struct TypedRefGuard<'a, T> {
  priv object_ref_guard: ObjectRefGuard<'a>
}

impl<'a, T> TypedRefGuard<'a, T> {
  /// Unlocks the guard, returning a reference to the ObjectRef that was used to
  /// create the guard originally.
  ///
  /// The guard is consumed in an attempt to prevent unintentional deadlocks due
  /// to double-locking within the task.
  pub fn unlock(self) -> &'a ObjectRef {
    self.object_ref_guard.unlock()
  }

  /// Compares the internal ObjectRef to another reference without unlocking.
  ///
  /// This can be used to check whether you already possess the guard to a given
  /// ObjectRef.
  pub fn ref_eq(&self, other: &ObjectRef) -> bool {
    self.object_ref_guard.ref_eq(other)
  }

  /// Converts this typed guard back into a regular, untyped `ObjectRefGuard`.
  pub fn into_untyped(self) -> ObjectRefGuard<'a> {
    self.object_ref_guard
  }
}

impl<'a, T:'static> Deref<T> for TypedRefGuard<'a, T> {
  fn deref<'a>(&'a self) -> &'a T {
    self.object_ref_guard.deref().as_any().as_ref::<T>().unwrap()
  }
}

impl<'a, T:'static> DerefMut<T> for TypedRefGuard<'a, T> {
  fn deref_mut<'a>(&'a mut self) -> &'a mut T {
    self.object_ref_guard.deref_mut().as_any_mut().as_mut::<T>().unwrap()
  }
}

/// A link to an object, to be referenced within an object's 'members' list.
#[deriving(Clone, Eq, TotalEq)]
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
  pub members: Vec<Option<Relationship>>,

  /// The Object's custom receiver, if present. See `Machine::combine()`.
  pub receiver: Option<ObjectRef>
}

impl Meta {
  /// Helpful constructor with some sensible default values.
  ///
  /// * `members`: empty vec
  /// * `receiver`: `None`
  pub fn new() -> Meta {
    Meta {
      members:  Vec::new(),
      receiver: None
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
  pub fn lookup_member(&self, key: &ObjectRef) -> Option<ObjectRef> {
    for maybe_relationship in self.members.tail().iter().rev() {
      match maybe_relationship {
        &Some(ref relationship) => {
          let object  = relationship.to().lock();
          let members = &object.deref().meta().members;

          if members.len() >= 3 {
            match (members.get(1), members.get(2)) {
              (&Some(ref rel_key), &Some(ref rel_value)) =>
                if rel_key.to().eq_as_symbol(key) ||
                   rel_key.to() == key {
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

/// The lowest level handler for a combination.
pub type NativeReceiver = fn (&Machine, Params) -> Reaction;

/// Parameters to be given to a receiver.
///
/// If the receiver were non-native, it would be sent these items as an empty
/// object with the members `[, caller, subject, message]`, so this structure
/// represents that without the overhead of constructing an object.
#[deriving(Clone, Eq, TotalEq)]
pub struct Params {
  /// The Execution-ish object from which the receiver was invoked.
  pub caller:  ObjectRef,

  /// The left-hand side of the combination that caused this receiver to be
  /// invoked.
  pub subject: ObjectRef,

  /// The right-hand side of the combination that caused this receiver to be
  /// invoked.
  pub message: ObjectRef
}

/// Indicates the result of a native operation exposed to the Paws-world, which
/// may either be an immediate realization (`React`) or delayed/non-existent
/// (`Yield`).
#[deriving(Clone, Eq, TotalEq)]
pub enum Reaction {
  /// Indicates that the reactor should realize the given execution and response
  /// immediately.
  ///
  /// 1. is the Execution-ish to realize (quite often the caller)
  /// 2. is the response to realize with
  React(ObjectRef, ObjectRef),

  /// Indicates that there is nothing that should be reacted immediately as a
  /// result of the receiver, so the reactor should wait on the Machine's queue
  /// instead.
  Yield
}
