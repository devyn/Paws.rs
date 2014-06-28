use object::symbol::*;

#[test]
fn create_symbol_and_retrieve_name() {
  let mut symbol_map = SymbolMap::new();

  let symbol = Symbol::new(symbol_map.intern("hello"));

  assert!(symbol.name() == "hello");
}

#[test]
fn compare_symbols() {
  let mut symbol_map = SymbolMap::new();

  let symbol1 = Symbol::new(symbol_map.intern("hello"));
  let symbol2 = Symbol::new(symbol_map.intern("hello"));
  let symbol3 = Symbol::new(symbol_map.intern("world"));

  assert!( symbol1.eq_by_name_ptr(&symbol2));
  assert!(!symbol1.eq_by_name_ptr(&symbol3));
}
