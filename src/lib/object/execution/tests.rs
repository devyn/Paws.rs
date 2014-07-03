use object::execution::*;

use machine::Machine;
use object::*;
use script::*;

#[test]
fn execution_advance_flat() {
  let machine = Machine::new();

  let symbol0 = machine.symbol("hello");
  let symbol1 = machine.symbol("world");

  let execution_ref =
    ObjectRef::new(
      box Execution::new(
        Script(vec![
          ObjectNode(symbol0.clone()),
          ObjectNode(symbol1.clone())
        ])
      ));

  let mut execution = execution_ref.lock().try_cast::<Execution>()
                        .ok().unwrap();

  let red   = machine.symbol("red");
  let green = machine.symbol("green");
  let blue  = machine.symbol("blue");

  // Pristine
  // {.hello world} advance(red)   => Combination(None <- hello)
  let combination0 =
    execution.advance(execution_ref.clone(), red.clone()).unwrap();

  assert!(combination0.subject.is_none());
  assert!(combination0.message == symbol0);

  // {hello .world} advance(green) => Combination(green <- world)
  let combination1 =
    execution.advance(execution_ref.clone(), green.clone()).unwrap();

  match combination1.subject {
    Some(ref object_ref) =>
      assert!(object_ref == &green),
    None => fail!("combination1.subject is None")
  }

  assert!(combination1.message == symbol1);

  // {hello world.} advance(blue)  => None
  assert!(execution.advance(execution_ref.clone(), blue).is_none());
}

#[test]
fn execution_advance_empty_expression() {
  let machine = Machine::new();

  let dummy_symbol = machine.symbol("dummy");

  let execution_ref =
    ObjectRef::new(box
      Execution::new(
        Script( vec![
          ExpressionNode( vec![] )] )));

  let mut execution = execution_ref.lock().try_cast::<Execution>()
                        .ok().unwrap();

  // Pristine
  // {.()} advance(dummy) => Combination(None <- <this>)
  let combination =
    execution.advance(execution_ref.clone(), dummy_symbol.clone()).unwrap();

  assert!(combination.subject.is_none());
  assert!(combination.message == execution_ref);
}

#[test]
fn execution_advance_nested_once() {
  let machine = Machine::new();

  let dummy = machine.symbol("dummy");
  let red   = machine.symbol("red");
  let green = machine.symbol("green");
  let blue  = machine.symbol("blue");

  let execution_ref =
    ObjectRef::new(box
      Execution::new(
        Script( vec![ObjectNode(dummy.clone()),
                     ExpressionNode( vec![ObjectNode(red.clone())] )] )));

  let mut execution = execution_ref.lock().try_cast::<Execution>()
                        .ok().unwrap();

  // Pristine
  // {.dummy (red)} advance(dummy) => Combination(None <- dummy)
  let combination0 =
    execution.advance(execution_ref.clone(), dummy.clone()).unwrap();

  assert!(combination0.subject.is_none());
  assert!(combination0.message == dummy);

  // {dummy .(red)} advance(green) => Combination(None <- red)
  // green {dummy (red.)}
  let combination1 =
    execution.advance(execution_ref.clone(), green.clone()).unwrap();

  assert!(combination1.subject.is_none());
  assert!(combination1.message == red);

  // green {dummy (red.)} advance(blue) => Combination(green <- blue)
  // {dummy (red)=blue.}
  let combination2 =
    execution.advance(execution_ref.clone(), blue.clone()).unwrap();

  match combination2.subject {
    Some(ref object_ref) =>
      assert!(object_ref == &green),
    None => fail!("combination2.subject is None")
  }

  assert!(combination2.message == blue);
}

#[test]
fn execution_advance_nested_twice() {
  let machine = Machine::new();

  let dummy = machine.symbol("dummy");
  let red   = machine.symbol("red");
  let green = machine.symbol("green");
  let blue  = machine.symbol("blue");
  let black = machine.symbol("black");

  let execution_ref =
    ObjectRef::new(box
      Execution::new(
        Script( vec![
          ObjectNode(dummy.clone()),
          ExpressionNode( vec![
            ExpressionNode( vec![ObjectNode(red.clone())] )] )] )));

  let mut execution = execution_ref.lock().try_cast::<Execution>()
                        .ok().unwrap();

  // Pristine
  // {.dummy ((red))} advance(dummy) => Combination(None <- dummy)
  let combination0 =
    execution.advance(execution_ref.clone(), dummy.clone()).unwrap();

  assert!(combination0.subject.is_none());
  assert!(combination0.message == dummy);

  // {dummy .((red))} advance(green) => Combination(None <- red)
  // green {dummy (.(red))}
  // green None {dummy ((.red))}
  // green None {dummy ((red.))}
  let combination1 =
    execution.advance(execution_ref.clone(), green.clone()).unwrap();

  assert!(combination1.subject.is_none());
  assert!(combination1.message == red);

  // green None {dummy ((red.))} advance(blue) => Combination(None <- blue)
  // green {dummy ((red)=blue.)}
  let combination2 =
    execution.advance(execution_ref.clone(), blue.clone()).unwrap();

  assert!(combination2.subject.is_none());
  assert!(combination2.message == blue);

  // green {dummy ((red)=blue.)} advance(black) => Combination(green <- black)
  // {dummy ((red)=blue)=black.}
  let combination3 =
    execution.advance(execution_ref.clone(), black.clone()).unwrap();

  match combination3.subject {
    Some(ref object_ref) =>
      assert!(object_ref == &green),
    None => fail!("combination3.subject is None")
  }

  assert!(combination3.message == black);
}
