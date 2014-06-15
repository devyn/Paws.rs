use object::*;
use object::symbol::*;

#[test]
fn create_symbol_and_retrieve_name() {
  let mut symbol_map = SymbolMap::new();

  let name = ~"hello";

  let symbol = Symbol::new(name, &mut symbol_map);

  // Try it twice
  assert!(symbol.name(&symbol_map) == &name);
  assert!(symbol.name(&symbol_map) == &name);
}

#[test]
fn compare_symbols() {
  let mut symbol_map = SymbolMap::new();

  let name1 = "hello";
  let name2 = "world";

  let symbol1 = Symbol::new(name1, &mut symbol_map);
  let symbol2 = Symbol::new(name1, &mut symbol_map);
  let symbol3 = Symbol::new(name2, &mut symbol_map);

  assert!(symbol1.equals_symbol(&symbol2));
  assert!(symbol2.equals_symbol(&symbol1));
  assert!(!symbol1.equals_symbol(&symbol3));
  assert!(!symbol2.equals_symbol(&symbol3));
}

#[test]
fn object_members() {
  let mut symbol_map = SymbolMap::new();

  let red   = ObjectRef::new(~Symbol::new("red",   &mut symbol_map));
  let green = ObjectRef::new(~Symbol::new("green", &mut symbol_map));
  let blue  = ObjectRef::new(~Symbol::new("blue",  &mut symbol_map));

  let mut red_object = red.write();

  red_object.members_mut().push(Relationship::new(green.clone()));
  red_object.members_mut().push(Relationship::new_child(blue.clone()));

  assert!( red_object.members().get(0).deref() == &green);
  assert!(!red_object.members().get(0).is_child());

  assert!( red_object.members().get(1).deref() == &blue);
  assert!( red_object.members().get(1).is_child());
}
