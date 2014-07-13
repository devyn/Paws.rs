//! Symbols are atoms that are interned into a machine-local table.

use object::*;

use std::io::IoResult;
use std::sync::Arc;
use std::collections::HashMap;

#[cfg(test)]
mod tests;

/// Maps strings to Symbol objects.
///
/// The most common usage is as part of a Machine.
#[deriving(Clone)]
pub struct SymbolMap {
  map: HashMap<String, Arc<String>>
}

impl SymbolMap {
  /// Creates an empty SymbolMap.
  pub fn new() -> SymbolMap {
    SymbolMap { map: HashMap::new() }
  }

  /// Returns a reference counted pointer to a string that is guaranteed to
  /// return the same pointer given two strings `a` and `b` if `a == b` is true.
  ///
  /// # Example
  ///
  ///     let mut symbol_map = SymbolMap::new();
  ///
  ///     // Intern four strings (`box` is used to guarantee uniqueness)
  ///     // Each pair is equivalent, but not pointer-equal.
  ///     // The results, however, will be.
  ///     let hello1 = symbol_map.intern(box "hello");
  ///     let hello2 = symbol_map.intern(box "hello");
  ///
  ///     let world1 = symbol_map.intern(box "world");
  ///     let world2 = symbol_map.intern(box "world");
  ///
  ///     // hello1 is pointer-equal to hello2
  ///     assert!((&*hello1 as *String) == (&*hello2 as *String));
  ///
  ///     // world1 is pointer-equal to world2
  ///     assert!((&*world1 as *String) == (&*world2 as *String));
  ///
  ///     // hello1 is NOT pointer-equal, however, to world1
  ///     assert!((&*hello1 as *String) != (&*world1 as *String));
  pub fn intern(&mut self, string: &str) -> Arc<String> {
    self.map.find_equiv(&string).map(|string_ptr| {

      string_ptr.clone()

    }).unwrap_or_else(|| {

      let string_ptr = Arc::new(string.to_string());

      self.map.insert(string.to_string(), string_ptr.clone());

      string_ptr

    })
  }
}

impl Collection for SymbolMap {
  fn len(&self) -> uint {
    // In case you want to know how many symbols have been interned.
    self.map.len()
  }
}

/// An object containing a string that should be comparable-by-pointer with
/// other `Symbol`s' strings from the same `SymbolMap`.
#[deriving(Clone)]
pub struct Symbol {
  name: Arc<String>,
  meta: Meta
}

impl Symbol {
  /// Creates a new Symbol object containing the given string Arc box.
  ///
  /// Note that `ObjectRef::new_symbol()` should be used instead of
  /// `ObjectRef::new()` when boxing this type up, in order to ensure that
  /// non-locking symbol comparison (`ObjectRef::eq_as_symbol()`) succeeds.
  pub fn new(name: Arc<String>) -> Symbol {
    Symbol {
      name: name,
      meta: Meta::new()
    }
  }

  /// The string that the symbol represents.
  pub fn name<'a>(&'a self) -> &'a str {
    self.name.as_slice()
  }

  /// Returns true if the Arc pointer in this Symbol points at the same string
  /// as the Arc pointer in the other Symbol.
  pub fn eq_by_name_ptr(&self, other: &Symbol) -> bool {
    (&*self.name as *String) == (&*other.name as *String)
  }

  /// Returns a new Arc pointing at the string that this Symbol contains.
  ///
  /// Prefer `name()` or `eq_by_name_ptr()` if applicable. Involves cloning an
  /// Arc which is less efficient than either for those purposes.
  pub fn name_ptr(&self) -> Arc<String> {
    self.name.clone()
  }
}

impl Object for Symbol {
  fn fmt_paws(&self, writer: &mut Writer) -> IoResult<()> {
    write!(writer, "Symbol[{}]", self.name())
  }

  fn meta<'a>(&'a self) -> &'a Meta {
    &self.meta
  }

  fn meta_mut<'a>(&'a mut self) -> &'a mut Meta {
    &mut self.meta
  }
}
