use super::*;

use script::*;

use machine::Machine;
use nuketype::Thing;

#[test]
fn advance_push_and_combine() {
  let machine = Machine::new();

  let symbol0 = machine.symbol("hello");
  let symbol1 = machine.symbol("world");

  let execution_ref =
    Execution::create(&machine,
      Script( vec![Discard,
                   Push(symbol0.clone()),
                   Push(symbol1.clone()),
                   Combine] ));

  let mut execution = execution_ref.lock().try_cast::<Execution>()
                        .ok().unwrap();

  let empty = Thing::empty();

  let combination = execution.advance(&execution_ref, empty).unwrap();

  assert!(combination.subject == Some(symbol0));
  assert!(combination.message == symbol1);
}

#[test]
fn advance_combine_locals_and_self() {
  let machine = Machine::new();

  let execution_ref =
    Execution::create(&machine,
      Script( vec![Discard,
                   PushLocals,
                   PushSelf,
                   Combine] ));

  let mut execution = execution_ref.lock().try_cast::<Execution>()
                        .ok().unwrap();

  let empty = Thing::empty();

  let combination =
    execution.advance(&execution_ref, empty).unwrap();

  assert!(combination.subject.is_none());
  assert!(combination.message == execution_ref);
}

#[test]
fn advance_elevated_push() {
  let machine = Machine::new();

  let dummy = machine.symbol("dummy");
  let red   = machine.symbol("red");
  let green = machine.symbol("green");
  let blue  = machine.symbol("blue");

  let execution_ref =
    Execution::create(&machine,
      Script( vec![Discard,
                   PushLocals,
                   Push(dummy.clone()),
                   Combine,

                   PushLocals,
                   Push(red.clone()),
                   Combine,

                   Combine] ));

  let mut execution = execution_ref.lock().try_cast::<Execution>()
                        .ok().unwrap();

  // Pristine
  // {.dummy (red)} advance(dummy) => Combination(None <- dummy)
  let combination0 =
    execution.advance(&execution_ref, dummy.clone()).unwrap();

  assert!(combination0.subject.is_none());
  assert!(combination0.message == dummy);

  // {dummy .(red)} advance(green) => Combination(None <- red)
  // green {dummy (red.)}
  let combination1 =
    execution.advance(&execution_ref, green.clone()).unwrap();

  assert!(combination1.subject.is_none());
  assert!(combination1.message == red);

  // green {dummy (red.)} advance(blue) => Combination(green <- blue)
  // {dummy (red)=blue.}
  let combination2 =
    execution.advance(&execution_ref, blue.clone()).unwrap();

  match combination2.subject {
    Some(ref object_ref) =>
      assert!(object_ref == &green),
    None => fail!("combination2.subject is None")
  }

  assert!(combination2.message == blue);
}

#[test]
fn creates_executions_that_have_unique_locals() {
  let machine = Machine::new();

  let execution1_ref = Execution::create(&machine, Script(vec![]));
  let execution2_ref = Execution::create(&machine, Script(vec![]));

  let execution1 = execution1_ref.lock().try_cast::<Execution>()
                     .ok().expect("not an Execution!");

  let execution2 = execution2_ref.lock().try_cast::<Execution>()
                     .ok().expect("not an Execution!");

  let locals1 = execution1.meta().members
                          .lookup_pair(&machine.symbol("locals"))
                          .expect("locals not found!");

  let locals2 = execution2.meta().members
                          .lookup_pair(&machine.symbol("locals"))
                          .expect("locals not found!");

  assert!(locals1 != locals2);
}
