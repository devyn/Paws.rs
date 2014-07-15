//! Paws objects, encapsulation, and metadata.

use std::any::*;
use std::sync::{Arc, Mutex, MutexGuard};
use std::io::IoResult;
use std::fmt::Show;
use std::fmt;

use machine::Machine;

pub use object::members::Members;

pub mod thing;
pub mod symbol;
pub mod execution;
pub mod alien;
pub mod locals;

mod members;

#[cfg(test)]
mod tests;

/// The interface that all Paws Objects must implement.
pub trait Object: Any {
  /// Formats a Paws Object for debugging purposes.
  fn fmt_paws(&self, writer: &mut Writer) -> IoResult<()>;

  /// Get access to the Object's metadata, including members and such.
  fn meta<'a>(&'a self) -> &'a Meta;

  /// Get mutable access to the Object's metadata.
  fn meta_mut<'a>(&'a mut self) -> &'a mut Meta;

  /// Converts an Object trait object to an Any trait object.
  ///
  /// You probably don't need to do this, as `AnyRefExt` is implemented for all
  /// `Object` references, which provides generic `is<T>()` and `as_ref<T>()`
  /// directly. It only exists in order to implement it.
  ///
  /// Additionally, `TypedRefGuard` exists, which is easier to use from an
  /// `ObjectRef`.
  fn as_any<'a>(&'a self) -> &'a Any {
    self as &Any
  }

  /// Same as `as_any()` but for a mutable ref.
  ///
  /// You probably don't need to do this, as `AnyMutRefExt` is implemented for
  /// all `Object` references, which provides generic `as_mut<T>()` directly. It
  /// only exists in order to implement it.
  ///
  /// Additionally, `TypedRefGuard` exists, which is easier to use from an
  /// `ObjectRef`.
  fn as_any_mut<'a>(&'a mut self) -> &'a mut Any {
    self as &mut Any
  }
}

impl<'a> AnyRefExt<'a> for &'a Object {
  fn is<T:'static>(self) -> bool {
    self.as_any().is::<T>()
  }

  fn as_ref<T:'static>(self) -> Option<&'a T> {
    self.as_any().as_ref::<T>()
  }
}

impl<'a> AnyMutRefExt<'a> for &'a mut Object {
  fn as_mut<T:'static>(self) -> Option<&'a mut T> {
    self.as_any_mut().as_mut::<T>()
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
  let lookup_result = {
    let subject = params.subject.lock();

    subject.deref().meta().members.lookup_pair(&params.message)
  };

  debug!("{} <lookup_receiver> {} => {}",
    params.subject, params.message, lookup_result);

  match lookup_result {
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
  i: Arc<ObjectRefInternal>
}

struct ObjectRefInternal {
  reference:  Mutex<Box<Object+Send+Share>>,

  /// For lockless symbol comparison.
  symbol_ref: Option<Arc<String>>,

  /// Allows tagging references, which makes debug output clearer.
  tag:        Option<Arc<String>>
}

impl ObjectRef {
  /// Boxes an Object trait into an Object reference.
  pub fn new(object: Box<Object+Send+Share>) -> ObjectRef {
    ObjectRef {
      i: Arc::new(ObjectRefInternal {
        reference:  Mutex::new(object),
        symbol_ref: None,
        tag:        None
      })
    }
  }

  /// Boxes an Object trait into an Object reference along with a tag for better
  /// debug output. Affects the result of the `Show` trait.
  pub fn new_with_tag<T: Tag>(
                      object: Box<Object+Send+Share>,
                      tag:    T)
                      -> ObjectRef {

    ObjectRef {
      i: Arc::new(ObjectRefInternal {
        reference:  Mutex::new(object),
        symbol_ref: None,
        tag:        tag.to_tag()
      })
    }
  }

  /// Boxes a Symbol into a Symbol reference.
  ///
  /// This is a special case to allow for lockless symbol comparison
  /// (`ObjectRef::eq_as_symbol()`). All Symbol-containing ObjectRefs are
  /// assumed to have been created this way; behavior is undefined if they are
  /// created with `ObjectRef::new()` instead.
  pub fn new_symbol(symbol: Box<symbol::Symbol>) -> ObjectRef {
    ObjectRef {
      i: Arc::new(ObjectRefInternal {
        symbol_ref: Some(symbol.name_ptr()),
        reference:  Mutex::new(symbol as Box<Object+Send+Share>),
        tag:        None
      })
    }
  }

  /// Obtain exclusive access to the Object this reference points to.
  ///
  /// The Object can be accessed via the returned RAII guard. The returned guard
  /// also contains a reference to this ObjectRef.
  pub fn lock<'a>(&'a self) -> ObjectRefGuard<'a> {
    ObjectRefGuard {
      object_ref: self,
      guard:      self.i.reference.lock()
    }
  }

  /// Returns true if both `ObjectRef`s are Symbol references that point at the
  /// same Symbol string.
  pub fn eq_as_symbol(&self, other: &ObjectRef) -> bool {
    match (&self.i.symbol_ref, &other.i.symbol_ref) {
      (&Some(ref a), &Some(ref b)) =>
        (&**a as *const String) == (&**b as *const String),

      _ => false
    }
  }

  /// If this `ObjectRef` is a Symbol reference, returns a reference to the
  /// pointer to the Symbol's name.
  pub fn symbol_ref<'a>(&'a self) -> Option<&'a Arc<String>> {
    self.i.symbol_ref.as_ref()
  }

  /// If this `ObjectRef` is a reference to something with a tag, returns a
  /// reference to the String representing the tag.
  pub fn tag<'a>(&'a self) -> Option<&'a Arc<String>> {
    self.i.tag.as_ref()
  }
}

impl PartialEq for ObjectRef {
  fn eq(&self, other: &ObjectRef) -> bool {
    (&*self.i  as *const ObjectRefInternal) ==
    (&*other.i as *const ObjectRefInternal)
  }
}

impl Eq for ObjectRef { }

impl Show for ObjectRef {
  fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
    match self.i.tag {
      Some(ref tag) =>
        write!(out, "[#{:p} ~{:s}]", &*self.i, tag.as_slice()),

      _ =>
        match self.i.symbol_ref {
          Some(ref string) =>
            write!(out, "[:{:s}]", string.as_slice()),
          None =>
            write!(out, "[#{:p}]", &*self.i)
        }
    }

  }
}

/// A trait to allow `ObjectRef::new_with_tag()` to be called with several
/// different natural values for tags. This means that both
///
///     ObjectRef::new_with_tag(box object, "my object")
///
/// and
///
///     ObjectRef::new_with_tag(box object, original.tag())
///
/// will work, among other variations.
pub trait Tag {
  /// Make an optional `Arc` to a `String`, which is the internal representation
  /// of a possible tag on an `ObjectRef`.
  ///
  /// It's wrapped in an `Option` so that we can use this with the result of
  /// `ObjectRef::tag()` directly.
  fn to_tag(&self) -> Option<Arc<String>>;
}

impl Tag for Arc<String> {
  fn to_tag(&self) -> Option<Arc<String>> {
    Some(self.clone())
  }
}

impl Tag for String {
  fn to_tag(&self) -> Option<Arc<String>> {
    Some(Arc::new(self.clone()))
  }
}

impl<'a> Tag for &'a str {
  fn to_tag(&self) -> Option<Arc<String>> {
    Some(Arc::new(self.to_string()))
  }
}

impl<'a, T: Tag> Tag for &'a T {
  fn to_tag(&self) -> Option<Arc<String>> {
    self.to_tag()
  }
}

impl<T: Tag> Tag for Option<T> {
  fn to_tag(&self) -> Option<Arc<String>> {
    self.as_ref().and_then(|t| t.to_tag())
  }
}

/// Represents exclusive access to an object.
///
/// Exclusive access is dropped when this guard is dropped.
pub struct ObjectRefGuard<'a> {
  object_ref: &'a ObjectRef,
  guard:      MutexGuard<'a, Box<Object+Send+Share>>
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
    if self.deref().is::<T>() {
      Ok(TypedRefGuard { object_ref_guard: self })
    } else {
      Err(self)
    }
  }
}

impl<'a> Deref<Box<Object+Send+Share>> for ObjectRefGuard<'a> {
  fn deref<'a>(&'a self) -> &'a Box<Object+Send+Share> {
    self.guard.deref()
  }
}

impl<'a> DerefMut<Box<Object+Send+Share>> for ObjectRefGuard<'a> {
  fn deref_mut<'a>(&'a mut self) -> &'a mut Box<Object+Send+Share> {
    self.guard.deref_mut()
  }
}

/// Allows pre-typechecked guards to be marked with their types to remove
/// redundant boilerplate when passing `ObjectRefGuard`s around.
pub struct TypedRefGuard<'a, T> {
  object_ref_guard: ObjectRefGuard<'a>
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
    self.object_ref_guard.deref().as_ref::<T>().unwrap()
  }
}

impl<'a, T:'static> DerefMut<T> for TypedRefGuard<'a, T> {
  fn deref_mut<'a>(&'a mut self) -> &'a mut T {
    self.object_ref_guard.deref_mut().as_mut::<T>().unwrap()
  }
}

/// A link to an object, to be referenced within an object's 'members' list.
#[deriving(Clone, Eq, PartialEq, Show)]
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

  /// The object this relationship points to.
  pub fn to<'a>(&'a self) -> &'a ObjectRef {
    &self.to
  }

  /// Turns the relationship into a 'child relationship'.
  pub fn own(&mut self) {
    self.is_child = true;
  }

  /// Turns the relationship into a 'non-child relationship'.
  pub fn disown(&mut self) {
    self.is_child = false;
  }

  /// Consumes the relationship, returning the object this relationship points
  /// to, whether it is a child relationship or not.
  pub fn unwrap(self) -> ObjectRef {
    self.to
  }
}

/// Object metadata -- this is universal for all objects, and required in order
/// to implement the `Object` trait.
#[deriving(Clone)]
pub struct Meta {
  /// A list of Relationships that make up the Object's members.
  ///
  /// Relationships that are 'children' are interpreted such that the Object
  /// 'owns' them, and so for responsibility to be acquired for this Object,
  /// responsibility must also be acquired for this Object's child members, and
  /// their child members, and so on.
  pub members:  Members,

  /// The Object's receiver (combination handler). See `Machine::combine()`.
  pub receiver: Receiver
}

impl Meta {
  /// Helpful constructor with some sensible default values.
  ///
  /// * **members**: empty
  /// * **receiver**: `NativeReceiver(lookup_receiver)`
  pub fn new() -> Meta {
    Meta {
      members:  Members::new(),
      receiver: NativeReceiver(lookup_receiver)
    }
  }

  /// Constructs a `Meta` exactly like `new()` does but with a given function to
  /// set as the receiver (`NativeReceiver`).
  pub fn with_receiver(receiver: fn (&Machine, Params) -> Reaction) -> Meta {
    let mut meta = Meta::new();

    meta.receiver = NativeReceiver(receiver);
    meta
  }
}

/// Specifies how a combination against an Object should be handled.
pub enum Receiver {
  /// If the object pointed to is queueable (an `Execution` or `Alien`), queue
  /// it with a `Params`-style Thing. Otherwise, look at its receiver
  /// recursively until a queueable or native receiver is found.
  ObjectReceiver(ObjectRef),

  /// Call this function immediately to get the appropriate `Reaction` to the
  /// combination, with the given `Machine` and `Params`.
  NativeReceiver(fn (&Machine, Params) -> Reaction)
}

impl Clone for Receiver {
  fn clone(&self) -> Receiver {
    match *self {
      ObjectReceiver(ref object_ref) =>
        ObjectReceiver(object_ref.clone()),

      NativeReceiver(function) =>
        NativeReceiver(function)
    }
  }
}

/// Parameters to be given to a receiver.
///
/// If the receiver were non-native, it would be sent these items as a Thing
/// with the members `[, caller, subject, message]`, so this structure
/// represents that without the overhead of constructing an object.
#[deriving(Clone, Eq, PartialEq, Show)]
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
#[deriving(Clone, Eq, PartialEq, Show)]
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
