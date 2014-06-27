//! Empties contain Object metadata and nothing more.

use object::*;

use std::io::IoResult;

/// A generic container type Object that is defined by only its metadata.
///
/// In other words: this just wraps a Meta and is the bare minimum necessary for
/// any Object implementation. It's actually very useful, because the bare
/// minimum is still quite a bit, and this lets you use that without any
/// specialized meaning attached.
#[deriving(Clone)]
pub struct Empty {
  priv meta: Meta
}

impl Empty {
  /// Creates a new Empty containing empty Meta (`Meta::new()`).
  pub fn new() -> Empty {
    Empty { meta: Meta::new() }
  }
}

impl Object for Empty {
  fn fmt_paws(&self, writer: &mut Writer) -> IoResult<()> {
    write!(writer, "Empty")
  }

  fn meta<'a>(&'a self) -> &'a Meta {
    &self.meta
  }

  fn meta_mut<'a>(&'a mut self) -> &'a mut Meta {
    &mut self.meta
  }
}
