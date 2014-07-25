//! Paws objects and metadata.

use nuketype::{Nuketype, Symbol};

use machine::reactor::Reactor;

use std::any::{AnyRefExt, AnyMutRefExt};

use std::hash::Hash;
use std::hash::sip::SipState;

use std::sync::{Arc, Weak, Mutex, MutexGuard};
use std::sync::atomics::{AtomicUint, SeqCst};

use std::fmt::Show;
use std::fmt;

pub use self::cache::Cache;
pub use self::members::Members;

pub mod cache;

mod members;

#[cfg(test)]
mod tests;

/// A receiver that simply calls `lookup_member()` on the subject's Meta with
/// the message as its argument.
///
/// If the lookup succeeds, the caller is staged with the result as the
/// response. If the lookup does not succeed, the caller is not re-staged.
///
/// This receiver is the default receiver for all Object types, unless
/// overridden.
pub fn lookup_receiver(reactor: &mut Reactor, params: Params) {
  // Use the local cache if we're looking up a symbol.
  let lookup_result =
    match params.message.symbol_ref() {
      Some(symbol) =>
        reactor.cache().sym_lookup(params.subject.clone(), symbol.clone()),

      None =>
        params.subject.lock().meta()
          .members.lookup_pair(&params.message)
    };

  debug!("{} <lookup_receiver> {} => {}",
    params.subject, params.message, lookup_result);

  match lookup_result {
    Some(value) =>
      reactor.stage(params.caller.clone(), value),
    None =>
      return
  }
}

/// A reference to an object. Use `lock()` to gain access to the data inside.
#[deriving(Clone)]
pub struct ObjectRef {
  reference: Arc<ObjectBox>
}

struct ObjectBox {
  data:         Mutex<ObjectData>,

  /// For lockless symbol comparison.
  symbol_ref:   Option<Arc<String>>,

  /// Allows tagging references, which makes debug output clearer.
  tag:          Option<Arc<String>>,

  /// For metadata caching.
  meta_version: AtomicUint,
}

struct ObjectData {
  nuketype: Box<Nuketype+Send+Share>,
  meta:     Meta
}

impl ObjectRef {
  /// Boxes a `Nuketype` and `Meta`, and returns a reference to that box.
  pub fn store(nuketype: Box<Nuketype+Send+Share>, meta: Meta) -> ObjectRef {
    ObjectRef::make(nuketype, meta, None, None)
  }

  /// Boxes a `Nuketype` and `Meta` along with a tag for better debug output.
  /// Affects the result of the `Show` trait.
  pub fn store_with_tag<T: Tag>(
                        nuketype: Box<Nuketype+Send+Share>,
                        meta:     Meta,
                        tag:      T)
                      -> ObjectRef {
    ObjectRef::make(nuketype, meta, None, tag.to_tag())
  }

  /// Boxes a `Symbol` nuketype into a symbol-reference.
  ///
  /// This is a special case to allow for lockless symbol comparison
  /// (`ObjectRef::eq_as_symbol()`). All `Symbol`-containing `ObjectRef`s are
  /// assumed to have been created this way; behavior is undefined if they are
  /// created with `ObjectRef::new()` instead.
  pub fn store_symbol(symbol: Box<Symbol>) -> ObjectRef {
    let symbol_ref = symbol.name_ptr();

    ObjectRef::make(symbol, Meta::new(), Some(symbol_ref), None)
  }

  fn make(nuketype:   Box<Nuketype+Send+Share>,
          meta:       Meta,
          symbol_ref: Option<Arc<String>>,
          tag:        Option<Arc<String>>)
          -> ObjectRef {

    ObjectRef {
      reference: Arc::new(ObjectBox {
        symbol_ref:   symbol_ref,
        tag:          tag,
        meta_version: AtomicUint::new(0),

        data: Mutex::new(ObjectData {
          nuketype: nuketype,
          meta:     meta
        })
      })
    }
  }

  /// Obtain exclusive access to the data this reference points to.
  ///
  /// The Nuketype and Meta can be accessed via the returned RAII guard. The
  /// returned guard also contains a reference to this ObjectRef.
  pub fn lock<'a>(&'a self) -> ObjectRefGuard<'a> {
    ObjectRefGuard {
      object_ref: self,
      guard:      self.reference.data.lock()
    }
  }

  /// Returns a new weak reference to the object that this reference points to.
  ///
  /// The weak reference will not keep the object alive, and so is suitable for
  /// caches.
  pub fn downgrade(&self) -> WeakObjectRef {
    WeakObjectRef {
      reference: self.reference.downgrade()
    }
  }

  /// Returns true if both `ObjectRef`s are Symbol references that point at the
  /// same Symbol string.
  pub fn eq_as_symbol(&self, other: &ObjectRef) -> bool {
    match (&self.reference.symbol_ref, &other.reference.symbol_ref) {
      (&Some(ref a), &Some(ref b)) =>
        (&**a as *const String) == (&**b as *const String),

      _ => false
    }
  }

  /// If this `ObjectRef` is a symbol-reference, returns a reference to the
  /// pointer to the Symbol's name.
  pub fn symbol_ref<'a>(&'a self) -> Option<&'a Arc<String>> {
    self.reference.symbol_ref.as_ref()
  }

  /// If this `ObjectRef` is a reference to something with a tag, returns a
  /// reference to the String representing the tag.
  pub fn tag<'a>(&'a self) -> Option<&'a Arc<String>> {
    self.reference.tag.as_ref()
  }

  /// Returns the metadata version of the object pointed to by this `ObjectRef`.
  ///
  /// The metadata version is automatically incremented whenever the object's
  /// metadata is modified, and can be used for caching metadata-related
  /// information.
  pub fn meta_version(&self) -> uint {
    self.reference.meta_version.load(SeqCst)
  }
}

impl PartialEq for ObjectRef {
  fn eq(&self, other: &ObjectRef) -> bool {
    (&*self.reference  as *const ObjectBox) ==
    (&*other.reference as *const ObjectBox)
  }
}

impl Eq for ObjectRef { }

impl Hash for ObjectRef {
  fn hash(&self, state: &mut SipState) {
    (&*self.reference as *const ObjectBox).hash(state)
  }
}

impl Show for ObjectRef {
  fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
    let _box: &ObjectBox = &*self.reference;

    match _box.tag {
      Some(ref tag) =>
        write!(out, "[#{:p} ~{:s}]", _box, tag.as_slice()),

      _ =>
        match _box.symbol_ref {
          Some(ref string) =>
            write!(out, "[:{:s}]", string.as_slice()),
          None =>
            write!(out, "[#{:p}]", _box)
        }
    }

  }
}

/// A weak reference to an object. Does not keep the object alive.
///
/// This should be used in caches to prevent the cache, which may live
/// considerably longer than many objects, from keeping objects alive.
///
/// Use `upgrade()` to get an `ObjectRef` if the object is still alive.
pub struct WeakObjectRef {
  reference: Weak<ObjectBox>
}

impl WeakObjectRef {
  /// Attempts to upgrade this weak reference to a strong reference.
  ///
  /// Returns `None` if the object is no longer alive.
  pub fn upgrade(&self) -> Option<ObjectRef> {
    self.reference.upgrade().map(|reference|
      ObjectRef {
        reference: reference
      }
    )
  }
}

/// A trait to allow `ObjectRef::store_with_tag()` to be called with several
/// different natural values for tags. This means that both
///
///     ObjectRef::store_with_tag(box object, meta, "my object")
///
/// and
///
///     ObjectRef::store_with_tag(box object, meta, original.tag())
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
    (**self).to_tag()
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
  object_ref:    &'a ObjectRef,
  guard:         MutexGuard<'a, ObjectData>
}

impl<'a> ObjectRefGuard<'a> {
  /// Get a reference to the guarded object's nuketype data.
  pub fn nuketype(&self) -> &Nuketype {
    // XXX: due to Rust bug
    let nuk_ref: &Nuketype = self.guard.deref().nuketype;
    nuk_ref
  }

  /// Get a mutable reference to the guarded object's nuketype data.
  ///
  /// Unless you intend to change the type of this object, you probably want
  /// to use `try_cast()` instead.
  pub fn nuketype_mut(&mut self) -> &mut Nuketype {
    // XXX: due to Rust bug
    let nuk_ref: &mut Nuketype = self.guard.deref_mut().nuketype;
    nuk_ref
  }

  /// Get a reference to the guarded object's metadata.
  pub fn meta(&self) -> &Meta {
    &self.guard.deref().meta
  }

  /// Get a mutable reference to the guarded object's metadata.
  ///
  /// Increments the metadata version of the object so that any metadata caches
  /// will be invalidated.
  pub fn meta_mut(&mut self) -> &mut Meta {
    self.object_ref.reference.meta_version.fetch_add(1, SeqCst);

    &mut self.guard.deref_mut().meta
  }

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
  /// Nuketype within matched the requested type, otherwise, returns this guard
  /// to be used again.
  pub fn try_cast<T:'static>(self)
                  -> Result<TypedRefGuard<'a, T>, ObjectRefGuard<'a>> {
    if self.nuketype().is::<T>() {
      Ok(TypedRefGuard { object_ref_guard: self })
    } else {
      Err(self)
    }
  }
}

/// Allows pre-typechecked guards to be marked with their Nuketypes' types to
/// remove redundant boilerplate when passing `ObjectRefGuard`s around.
pub struct TypedRefGuard<'a, T> {
  object_ref_guard: ObjectRefGuard<'a>
}

impl<'a, T> TypedRefGuard<'a, T> {
  /// Get a reference to the guarded object's metadata.
  pub fn meta(&self) -> &Meta {
    self.object_ref_guard.meta()
  }

  /// Get a mutable reference to the guarded object's metadata.
  pub fn meta_mut(&mut self) -> &mut Meta {
    self.object_ref_guard.meta_mut()
  }

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
    self.object_ref_guard.nuketype().as_ref::<T>().unwrap()
  }
}

impl<'a, T:'static> DerefMut<T> for TypedRefGuard<'a, T> {
  fn deref_mut<'a>(&'a mut self) -> &'a mut T {
    self.object_ref_guard.nuketype_mut().as_mut::<T>().unwrap()
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
  pub fn with_receiver(receiver: fn (&mut Reactor, Params)) -> Meta {
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

  /// Call this function immediately to perform the combination, with the given
  /// `Reactor` and `Params`.
  NativeReceiver(fn (&mut Reactor, Params))
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
