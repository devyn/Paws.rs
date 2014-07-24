//! Nuclear types: implementation-defined opaque data.
//!
//! Paws Nucleus traditionally has three nuketypes:
//!
//! * **Thing** (represented by `Thing`, and, in one case, `Locals`)
//! * **Label** (represented by `Symbol`)
//! * **Execution** (represented by `Execution` and `Alien` for transparent and
//!   opaque variants, respectively)

use std::any::{Any, AnyRefExt, AnyMutRefExt};

use std::io::IoResult;

pub use self::thing::Thing;
pub use self::symbol::Symbol;
pub use self::execution::Execution;
pub use self::alien::Alien;
pub use self::locals::Locals;

pub mod thing;
pub mod symbol;
pub mod execution;
pub mod alien;
pub mod locals;

/// The interface that all Nuclear types ("nuketypes") must implement.
pub trait Nuketype: Any {
  /// Formats a Paws Object for debugging purposes.
  fn fmt_paws(&self, writer: &mut Writer) -> IoResult<()>;

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

impl<'a> AnyRefExt<'a> for &'a Nuketype {
  fn is<T:'static>(self) -> bool {
    self.as_any().is::<T>()
  }

  fn as_ref<T:'static>(self) -> Option<&'a T> {
    self.as_any().as_ref::<T>()
  }
}

impl<'a> AnyMutRefExt<'a> for &'a mut Nuketype {
  fn as_mut<T:'static>(self) -> Option<&'a mut T> {
    self.as_any_mut().as_mut::<T>()
  }
}
