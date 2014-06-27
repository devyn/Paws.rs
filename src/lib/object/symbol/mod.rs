//! Paws symbols are atoms that are interned into a global table.

use object::*;

use std::io::IoResult;
use collections::hashmap::HashMap;

#[cfg(test)]
mod tests;

/// Maps strings to Symbol objects.
///
/// The most common usage is as part of a Machine.
#[deriving(Clone)]
pub struct SymbolMap {
  priv map: HashMap<~str, ObjectRef>
}

impl SymbolMap {
  /// Creates an empty SymbolMap.
  pub fn new() -> SymbolMap {
    SymbolMap { map: HashMap::new() }
  }

  /// Returns a reference to the Symbol whose name matches the given string.
  ///
  /// Creates the Symbol and adds it to the map if it doesn't exist. This is an
  /// intentionally leaky and irreversable operation.
  pub fn intern(&mut self, string: &str) -> ObjectRef {
    self.map.find_equiv(&string).map(|symbol| {

      symbol.clone()

    }).unwrap_or_else(|| {

      let symbol = ObjectRef::new(~Symbol {
                     name: string.to_owned(),
                     meta: Meta::new()
                   });

      self.map.insert(string.to_owned(), symbol.clone());

      symbol

    })
  }
}

impl Container for SymbolMap {
  fn len(&self) -> uint {
    // In case you want to know how many symbols have been interned.
    self.map.len()
  }
}

/// An object containing a string that should be comparable-by-pointer with
/// other `Symbol`s from the same `SymbolMap`.
///
/// There is no constructor (i.e. `new()`) function, because they are intended
/// to be created on-demand by a `SymbolMap`.
#[deriving(Clone)]
pub struct Symbol {
  priv name: ~str,
  priv meta: Meta
}

impl Symbol {
  /// The string that the symbol represents.
  pub fn name<'a>(&'a self) -> &'a str {
    self.name.as_slice()
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
