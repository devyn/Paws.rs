use object::execution::*;

use machine::Machine;
use object::*;
use script::*;

#[test]
fn execution_advance_flat() {
  let mut machine = Machine::new();

  let symbols: ~[ObjectRef] = ["hello", "world"].iter().map(|&string|
    machine.symbol(string)
  ).collect();

  let execution_ref =
    ObjectRef::new(
      ~Execution::new(
        Script(symbols.iter().map(|object_ref|
          ObjectNode(object_ref.clone())
        ).collect())
      ));

  let mut execution = execution_ref.lock().try_cast::<Execution>().unwrap();

  let red   = machine.symbol("red");
  let green = machine.symbol("green");
  let blue  = machine.symbol("blue");

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

  let dummy_symbol = machine.symbol("dummy");

  let execution_ref =
    ObjectRef::new(
      ~Execution::new(
        Script(~[ExpressionNode(~[])])));

  let mut execution = execution_ref.lock().try_cast::<Execution>().unwrap();

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

  let red   = machine.symbol("red");
  let green = machine.symbol("green");
  let blue  = machine.symbol("blue");

  let execution_ref =
    ObjectRef::new(
      ~Execution::new(
        Script(~[ExpressionNode(~[ObjectNode(red.clone())])])));

  let mut execution = execution_ref.lock().try_cast::<Execution>().unwrap();

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

  let red   = machine.symbol("red");
  let green = machine.symbol("green");
  let blue  = machine.symbol("blue");
  let black = machine.symbol("black");

  let execution_ref =
    ObjectRef::new(
      ~Execution::new(
        Script(~[
          ExpressionNode(~[
            ExpressionNode(~[ObjectNode(red.clone())])])])));

  let mut execution = execution_ref.lock().try_cast::<Execution>().unwrap();

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
