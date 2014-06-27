use object::symbol::*;

#[test]
fn create_symbol_and_retrieve_name() {
  let mut symbol_map = SymbolMap::new();

  let symbol = symbol_map.intern("hello");

  assert!(symbol.lock().try_cast::<Symbol>().unwrap().name() == "hello");
}

#[test]
fn compare_symbols() {
  let mut symbol_map = SymbolMap::new();

  let symbol1 = symbol_map.intern("hello");
  let symbol2 = symbol_map.intern("hello");
  let symbol3 = symbol_map.intern("world");

  assert!(symbol1 == symbol2);
  assert!(symbol1 != symbol3);
}
