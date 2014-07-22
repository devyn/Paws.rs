use machine::*;
use script::*;

use object::*;
use object::execution::Execution;

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

#[test]
fn machine_creates_executions_that_have_unique_locals() {
  let machine = Machine::new();

  let execution1_ref = machine.execution(Script(vec![]));
  let execution2_ref = machine.execution(Script(vec![]));

  let execution1 = execution1_ref.lock().try_cast::<Execution>()
                     .ok().expect("not an Execution!");

  let execution2 = execution2_ref.lock().try_cast::<Execution>()
                     .ok().expect("not an Execution!");

  let locals1 = execution1.deref().meta().members
                          .lookup_pair(&machine.symbol("locals"))
                          .expect("locals not found!");

  let locals2 = execution2.deref().meta().members
                          .lookup_pair(&machine.symbol("locals"))
                          .expect("locals not found!");

  assert!(locals1 != locals2);
}
