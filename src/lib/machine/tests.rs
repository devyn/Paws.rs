use machine::*;
use script::*;

use object::*;
use object::alien::Alien;
use object::thing::Thing;
use object::execution::Execution;

use std::any::AnyRefExt;

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

#[test]
fn machine_can_combine_via_direct_default_receiver() {
  let machine = Machine::new();

  let caller_ref  = machine.execution(Script(vec![]));
  let message_ref = ObjectRef::new(box Thing::new());

  // This might seem a little unclear, but effectively what we're doing here is
  // combining the caller itself with a target message. Could be a real-world
  // scenario; I don't know.
  //
  // Anyway, I chose this because the `stage_receiver` is really the simplest of
  // all of them to target, which is what an Execution has.
  let reaction = machine.combine(caller_ref.lock(), Combination {
    subject: Some(caller_ref.clone()),
    message: message_ref.clone()
  });

  match reaction {
    React(execution, response) => {
      assert!(message_ref == response);

      assert!(caller_ref != execution);
      assert!(caller_ref.lock().try_cast::<Execution>().ok().unwrap().root() ==
               execution.lock().try_cast::<Execution>().ok().unwrap().root());
    },
    _ => fail!("unexpected {}, expected React", reaction)
  }
}

#[test]
fn machine_can_combine_via_indirect_default_receiver() {
  let machine = Machine::new();

  let caller_ref = machine.execution(Script(vec![]));
  let other_ref  = ObjectRef::new(box Thing::new());
  let key_ref    = ObjectRef::new(box Thing::new());
  let value_ref  = ObjectRef::new(box Thing::new());

  {
    // The goal here is to use the other's `lookup_receiver` on the caller to
    // look up the caller's member. This is a pretty real world thing to do,
    // although you probably wouldn't want to change an Execution's receiver;
    // instead, you would want to take the `lookup_receiver` as an alien and use
    // it.
    let mut caller = caller_ref.lock();
    
    caller.meta_mut().receiver = ObjectReceiver(other_ref);

    caller.meta_mut().members.push_pair_to_child(
      key_ref.clone(), value_ref.clone());
  }

  let reaction = machine.combine(caller_ref.lock(), Combination {
    subject: Some(caller_ref.clone()),
    message: key_ref
  });

  assert!(reaction == React(caller_ref, value_ref));
}

#[test]
fn machine_can_combine_via_executionish_receiver() {
  let machine = Machine::new();

  #[allow(unused_variable)]
  fn stub_routine<'a>(
                  alien: TypedRefGuard<'a, Alien>,
                  machine: &Machine,
                  response: ObjectRef)
                  -> Reaction {

    fail!("stub_routine was called!")
  }

  #[deriving(Clone)]
  struct StubData;

  let stub_data     = box StubData;

  let caller_ref    = machine.execution(Script(vec![]));
  let execution_ref = machine.execution(Script(vec![]));
  let alien_ref     = ObjectRef::new(box Alien::new(stub_routine, stub_data));
  let other_ref     = ObjectRef::new(box Thing::new());
  let message_ref   = ObjectRef::new(box Thing::new());

  // We have to try two things here: changing receiver to an Execution, and
  // changing receiver to an Alien. `other_ref` will be our target.
  //
  // Both reactions should be the similar: reacting the chosen receiver with a
  // params object `[, caller, other, message]`

  let check_reaction = |receiver| {
    let reaction = machine.combine(caller_ref.lock(), Combination {
      subject: Some(other_ref.clone()),
      message: message_ref.clone()
    });

    match reaction {
      React(execution, response_ref) => {
        assert!(execution != receiver);

        match execution.lock().try_cast::<Execution>() {
          Ok(execution) =>
            assert!(
              execution.deref().root() ==
              receiver.lock().try_cast::<Execution>().ok().unwrap().root()),
          Err(unknown) =>
            match unknown.try_cast::<Alien>() {
              Ok(alien) =>
                assert!(alien.deref().data.is::<StubData>()),
              Err(_) =>
                fail!("Object being staged is neither Execution nor Alien")
            }
        }

        let response = response_ref.lock();

        let members = &response.deref().meta().members;

        // Match with `[, caller, other, message]`
        assert!(members.get(0).is_none());

        assert!(members.get(1) ==
                Some(&Relationship::new(caller_ref.clone())));

        assert!(members.get(2) ==
                Some(&Relationship::new(other_ref.clone())));

        assert!(members.get(3) ==
                Some(&Relationship::new(message_ref.clone())));
      },

      Yield => fail!("expected React(...), got Yield")
    }
  };

  other_ref.lock().meta_mut().receiver = ObjectReceiver(execution_ref.clone());

  check_reaction(execution_ref);

  other_ref.lock().meta_mut().receiver = ObjectReceiver(alien_ref.clone());

  check_reaction(alien_ref);
}

#[test]
fn machine_can_combine_with_and_lookup_on_implicit_locals() {
  let machine = Machine::new();

  let caller_ref = machine.execution(Script(vec![]));

  let key_ref    = ObjectRef::new(box Thing::new());
  let value_ref  = ObjectRef::new(box Thing::new());

  {
    // Add a key and value to the caller's locals.
    let caller     = caller_ref.lock();

    let locals_ref = caller.deref().meta().members
                           .lookup_pair(&machine.symbol("locals"))
                           .expect("locals not found on created Execution!");

    let mut locals = locals_ref.lock();

    locals.meta_mut().members.push_pair_to_child(
      key_ref.clone(), value_ref.clone());
  }

  let reaction = machine.combine(caller_ref.lock(), Combination {
    subject: None,
    message: key_ref
  });

  assert!(reaction == React(caller_ref, value_ref));
}

#[test]
fn machine_react_stop_call() {
  let machine = Machine::new();

  let caller_ref = machine.execution(
                     Script(vec![
                       ObjectNode(machine.symbol("stop")),
                       ExpressionNode(vec![])]));

  #[allow(unused_variable)]
  fn stop_routine<'a>(
                  alien: TypedRefGuard<'a, Alien>,
                  machine: &Machine,
                  response: ObjectRef)
                  -> Reaction {

    machine.stop();
    Yield
  }

  let stop_alien_ref = ObjectRef::new(box Alien::new(
                         stop_routine, box() ()));

  {
    // Affix a stop alien onto the caller's locals.
    let caller     = caller_ref.lock();

    let locals_ref = caller.deref().meta().members
                           .lookup_pair(&machine.symbol("locals"))
                           .expect("locals not found on created Execution!");

    let mut locals = locals_ref.lock();

    locals.meta_mut().members.push_pair_to_child(
      machine.symbol("stop"), stop_alien_ref);
  }

  // Almost ready...
  //
  // Since it's pristine we can really give it anything we want.
  machine.enqueue(caller_ref.clone(), caller_ref);

  // Let's go!
  machine.run_reactor();
}
