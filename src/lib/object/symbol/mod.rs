//! Paws symbols are atoms that are interned into a global table.

use std::hash;
use collections::treemap::TreeMap;
use object::Object;

#[cfg(test)]
mod tests;

/// Holds a map of 64-bit keys to symbol names.
///
/// All `Symbol`s' keys reference into a map and are created paired to a
/// specific map, so it is important to keep track of the `SymbolMap` used to
/// create a given `Symbol`.
pub struct SymbolMap(TreeMap<u64, ~str>);

impl SymbolMap {
  /// Creates an empty SymbolMap.
  pub fn new() -> SymbolMap {
    SymbolMap(TreeMap::new())
  }

  /// Hashes the symbol string as its `key` and returns it.
  ///
  /// Also creates an entry in the map associating the key with the symbol
  /// string if one doesn't already exist so that it can be looked up later.
  pub fn intern(&mut self, symbol: &str) -> u64 {
    let SymbolMap(ref mut tree_map) = *self;

    let key = hash::hash(&symbol);

    match tree_map.find(&key) {
      Some(_) => (),
      None    => {
        tree_map.swap(key, symbol.to_owned());
      }
    }

    key
  }
}

impl Container for SymbolMap {
  fn len(&self) -> uint {
    let SymbolMap(ref tree_map) = *self;
    tree_map.len()
  }
}

impl Map<u64, ~str> for SymbolMap {
  fn find<'a>(&'a self, key: &u64) -> Option<&'a ~str> {
    let SymbolMap(ref tree_map) = *self;
    tree_map.find(key)
  }
}

/// Holds a key to reference into a given `SymbolMap`.
pub struct Symbol {
  key: u64
}

impl Symbol {
  /// Creates a symbol by interning it in a `SymbolMap`.
  pub fn new(name: &str, symbol_map: &mut SymbolMap) -> Symbol {
    Symbol {
      key: symbol_map.intern(name)
    }
  }

  /// Looks up the name of the symbol in the given `SymbolMap`.
  ///
  /// Using a `SymbolMap` other than the one used to create the `Symbol` may
  /// result in a task failure, or worse, a mismatched name.
  pub fn name<'a>(&self, symbol_map: &'a SymbolMap) -> &'a ~str {
    symbol_map.find(&self.key).expect("symbol not in map")
  }

  /// Compares this symbol against another symbol by key only.
  ///
  /// Take care to ensure the `Symbol`s are from the same `SymbolMap`,
  /// otherwise, the result is undefined.
  pub fn equals_symbol(&self, other: &Symbol) -> bool {
    self.key == other.key
  }
}

impl Object for Symbol {
}
