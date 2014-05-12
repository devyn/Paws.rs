//! Paws machines and reactor implementation.

use object::symbol::SymbolMap;

pub struct Machine {
  pub symbol_map: SymbolMap
}

impl Machine {
  pub fn new() -> Machine {
    Machine {
      symbol_map: SymbolMap::new()
    }
  }
}
