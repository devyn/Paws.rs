use super::Machine;

#[test]
fn machine_creates_symbols_with_different_object_identity() {
  let machine = Machine::new();

  assert!(machine.symbol("foo") != machine.symbol("foo"));
}

#[test]
fn machine_can_create_symbols_that_match() {
  let machine = Machine::new();

  assert!(machine.symbol("foo").eq_as_symbol(&machine.symbol("foo")));
}

#[test]
fn machine_can_create_symbols_that_dont_match() {
  let machine = Machine::new();

  assert!(!machine.symbol("foo").eq_as_symbol(&machine.symbol("bar")));
}
