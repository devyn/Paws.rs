use object::execution::*;

use machine::Machine;
use object::*;
use script::*;

use std::any::*;

#[test]
fn execution_advance_flat() {
  let mut machine = Machine::new();

  let symbols: ~[ObjectRef] = ["hello", "world"].iter().map(|&string|
    ObjectRef::new(~machine.symbol(string))
  ).collect();

  let execution_ref =
    ObjectRef::new(
      ~Execution::new(
        Script(symbols.iter().map(|object_ref|
          ObjectNode(object_ref.clone())
        ).collect())
      ));

  let mut execution_ref_borrow = execution_ref.borrow_mut();

  let execution: &mut Execution =
    execution_ref_borrow.as_any_mut().as_mut().unwrap();

  let red   = ObjectRef::new(~machine.symbol("red"));
  let green = ObjectRef::new(~machine.symbol("green"));
  let blue  = ObjectRef::new(~machine.symbol("blue"));

  // {.hello world} advance(red)   => Combination(red <- hello)
  let combination0 =
    execution.advance(execution_ref.clone(), red.clone()).unwrap();

  match combination0.subject {
    Some(ref object_ref) =>
      assert!(object_ref == &red),
    None => fail!("combination0.subject is None")
  }

  assert!(combination0.message == symbols[0]);

  // {hello .world} advance(green) => Combination(green <- world)
  let combination1 =
    execution.advance(execution_ref.clone(), green.clone()).unwrap();

  match combination1.subject {
    Some(ref object_ref) =>
      assert!(object_ref == &green),
    None => fail!("combination1.subject is None")
  }

  assert!(combination1.message == symbols[1]);

  // {hello world.} advance(blue)  => None
  assert!(execution.advance(execution_ref.clone(), blue).is_none());
}

#[test]
fn execution_advance_empty_expression() {
  let mut machine = Machine::new();

  let dummy_symbol =
    ObjectRef::new(~machine.symbol("dummy"));

  let execution_ref =
    ObjectRef::new(
      ~Execution::new(
        Script(~[ExpressionNode(~[])])));

  let mut execution_ref_borrow = execution_ref.borrow_mut();

  let execution: &mut Execution =
    execution_ref_borrow.as_any_mut().as_mut().unwrap();

  // {.()} advance(dummy) => Combination(dummy <- <this>)
  let combination =
    execution.advance(execution_ref.clone(), dummy_symbol.clone()).unwrap();

  match combination.subject {
    Some(ref object_ref) =>
      assert!(object_ref == &dummy_symbol),
    None => fail!("combination.subject is None")
  }

  assert!(combination.message == execution_ref);
}

#[test]
fn execution_advance_nested_once() {
  let mut machine = Machine::new();

  let red   = ObjectRef::new(~machine.symbol("red"));
  let green = ObjectRef::new(~machine.symbol("green"));
  let blue  = ObjectRef::new(~machine.symbol("blue"));

  let execution_ref =
    ObjectRef::new(
      ~Execution::new(
        Script(~[ExpressionNode(~[ObjectNode(red.clone())])])));

  let mut execution_ref_borrow = execution_ref.borrow_mut();

  let execution: &mut Execution =
    execution_ref_borrow.as_any_mut().as_mut().unwrap();

  // {.(red)} advance(green) => Combination(None <- red)
  // green {(red.)}
  let combination0 =
    execution.advance(execution_ref.clone(), green.clone()).unwrap();

  assert!(combination0.subject.is_none());
  assert!(combination0.message == red);

  // green {(red.)} advance(blue) => Combination(green <- blue)
  // {(red)=blue.}
  let combination1 =
    execution.advance(execution_ref.clone(), blue.clone()).unwrap();

  match combination1.subject {
    Some(ref object_ref) =>
      assert!(object_ref == &green),
    None => fail!("combination1.subject is None")
  }

  assert!(combination1.message == blue);
}

#[test]
fn execution_advance_nested_twice() {
  let mut machine = Machine::new();

  let red   = ObjectRef::new(~machine.symbol("red"));
  let green = ObjectRef::new(~machine.symbol("green"));
  let blue  = ObjectRef::new(~machine.symbol("blue"));
  let black = ObjectRef::new(~machine.symbol("black"));

  let execution_ref =
    ObjectRef::new(
      ~Execution::new(
        Script(~[
          ExpressionNode(~[
            ExpressionNode(~[ObjectNode(red.clone())])])])));

  let mut execution_ref_borrow = execution_ref.borrow_mut();

  let execution: &mut Execution =
    execution_ref_borrow.as_any_mut().as_mut().unwrap();

  // {.((red))} advance(green) => Combination(None <- red)
  // green {(.(red))}
  // green None {((.red))}
  // green None {((red.))}
  let combination0 =
    execution.advance(execution_ref.clone(), green.clone()).unwrap();

  assert!(combination0.subject.is_none());
  assert!(combination0.message == red);

  // green None {((red.))} advance(blue) => Combination(None <- blue)
  // green {((red)=blue.)}
  let combination1 =
    execution.advance(execution_ref.clone(), blue.clone()).unwrap();

  assert!(combination1.subject.is_none());
  assert!(combination1.message == blue);

  // green {((red)=blue.)} advance(black) => Combination(green <- black)
  // {((red)=blue)=black.}
  let combination2 =
    execution.advance(execution_ref.clone(), black.clone()).unwrap();

  match combination2.subject {
    Some(ref object_ref) =>
      assert!(object_ref == &green),
    None => fail!("combination2.subject is None")
  }

  assert!(combination2.message == black);
}

#[test]
fn object_members() {
  let mut machine = Machine::new();

  let execution_ref = ObjectRef::new(~Execution::new(Script(~[])));

  let red   = ObjectRef::new(~machine.symbol("red"));
  let green = ObjectRef::new(~machine.symbol("green"));

  let mut execution_object = execution_ref.borrow_mut();

  execution_object.members_mut().push(Relationship::new(red.clone()));
  execution_object.members_mut().push(Relationship::new_child(green.clone()));

  assert!( execution_object.members().get(0).deref() == &red);
  assert!(!execution_object.members().get(0).is_child());

  assert!( execution_object.members().get(1).deref() == &green);
  assert!( execution_object.members().get(1).is_child());
}
