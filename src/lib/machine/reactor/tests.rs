use super::*;

use script::*;

use object::{ObjectRef, Relationship, ObjectReceiver};
use object::TypedRefGuard;

use nuketype::{Alien, Thing, Execution};

use machine::Machine;

use util;

use std::any::AnyRefExt;

#[test]
fn combine_via_direct_default_receiver() {
  let     machine = Machine::new();
  let mut reactor = MockReactor::new(machine.clone());

  let caller_ref  = Execution::create(&machine, Script(vec![]));
  let message_ref = Thing::empty();

  // This might seem a little unclear, but effectively what we're doing here is
  // combining the caller itself with a target message. Could be a real-world
  // scenario; I don't know.
  combine(&mut reactor, caller_ref.lock(), Combination {
    subject: Some(caller_ref.clone()),
    message: message_ref.clone()
  });

  match reactor.stagings.remove(0) {
    Some((execution, response)) => {
      assert!(message_ref == response);

      assert!(caller_ref != execution);
      assert!(caller_ref.lock().try_cast::<Execution>().ok().unwrap().root() ==
               execution.lock().try_cast::<Execution>().ok().unwrap().root());
    },
    None => fail!("unexpected end of queue")
  }
}

#[test]
fn combine_via_indirect_default_receiver() {
  let     machine = Machine::new();
  let mut reactor = MockReactor::new(machine.clone());

  let caller_ref = Execution::create(&machine, Script(vec![]));
  let other_ref  = Thing::empty();
  let key_ref    = Thing::empty();
  let value_ref  = Thing::empty();

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

  combine(&mut reactor, caller_ref.lock(), Combination {
    subject: Some(caller_ref.clone()),
    message: key_ref
  });

  assert!(reactor.stagings.shift() == Some((caller_ref, value_ref)));
}

#[test]
fn combine_via_executionish_receiver() {
  let     machine = Machine::new();
  let mut reactor = MockReactor::new(machine.clone());

  fn stub_routine<'a>(
                  _alien: TypedRefGuard<'a, Alien>,
                  _reactor: &mut Reactor,
                  _response: ObjectRef) {

    fail!("stub_routine was called!")
  }

  #[deriving(Clone)]
  struct StubData;

  let stub_data     = box StubData;

  let caller_ref    = Execution::create(&machine, Script(vec![]));
  let other_ref     = Thing::empty();
  let message_ref   = Thing::empty();

  let execution_ref = Execution::create(&machine, Script(vec![]));
  let alien_ref     = Alien::create("stub", stub_routine, stub_data);

  // We have to try two things here: changing receiver to an Execution, and
  // changing receiver to an Alien. `other_ref` will be our target.
  //
  // Both reactions should be the similar: reacting the chosen receiver with a
  // params object `[, caller, other, message]`

  for receiver in [execution_ref, alien_ref].iter() {
    other_ref.lock().meta_mut().receiver = ObjectReceiver(receiver.clone());

    combine(&mut reactor, caller_ref.lock(), Combination {
      subject: Some(other_ref.clone()),
      message: message_ref.clone()
    });

    match reactor.stagings.remove(0) {
      Some((execution, response_ref)) => {
        assert!(&execution != receiver);

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

        let members = &response.meta().members;

        // Match with `[, caller, other, message]`
        assert!(members.get(0).is_none());

        assert!(members.get(1) ==
                Some(&Relationship::new(caller_ref.clone())));

        assert!(members.get(2) ==
                Some(&Relationship::new(other_ref.clone())));

        assert!(members.get(3) ==
                Some(&Relationship::new(message_ref.clone())));
      },

      None => fail!("staging queue is empty")
    }
  };
}

#[test]
fn combine_with_and_lookup_on_implicit_locals() {
  let     machine = Machine::new();
  let mut reactor = MockReactor::new(machine.clone());

  let caller_ref = Execution::create(&machine, Script(vec![]));

  let key_ref    = Thing::empty();
  let value_ref  = Thing::empty();

  {
    // Add a key and value to the caller's locals.
    let caller     = caller_ref.lock();

    let locals_ref = caller.meta().members
                           .lookup_pair(&machine.symbol("locals"))
                           .expect("locals not found on created Execution!");

    let mut locals = locals_ref.lock();

    locals.meta_mut().members.push_pair_to_child(
      key_ref.clone(), value_ref.clone());
  }

  combine(&mut reactor, caller_ref.lock(), Combination {
    subject: None,
    message: key_ref
  });

  assert!(reactor.stagings.shift() == Some((caller_ref, value_ref)));
}

struct ReactorTest {
  init: proc(&mut Reactor): Send,
  fini: proc(): Send
}

fn test_reactor_stall_handlers() -> ReactorTest {
  let (stalled_tx, stalled_rx) = channel::<()>();

  ReactorTest {
    init: proc(reactor) {
      reactor.on_stall(proc (reactor) {
        stalled_tx.send(());
        reactor.stop();
      });
    },
    fini: proc() {
      assert!(stalled_rx.try_recv().is_ok());
    }
  }
}

fn test_reactor_react_stop_call(machine: &Machine) -> ReactorTest {

  let caller_ref = Execution::create(machine,
                     Script(vec![Discard,
                                 PushLocals,
                                 Push(machine.symbol("stop")),
                                 Combine,
                                 PushSelf,
                                 Combine]));

  fn stop_routine<'a>(
                  _alien: TypedRefGuard<'a, Alien>,
                   reactor: &mut Reactor,
                  _response: ObjectRef) {

    reactor.stop();
  }

  let stop_alien_ref = Alien::create("stop", stop_routine, box() ());

  {
    // Affix a stop alien onto the caller's locals.
    let caller     = caller_ref.lock();

    let locals_ref = caller.meta().members
                           .lookup_pair(&machine.symbol("locals"))
                           .expect("locals not found on created Execution!");

    let mut locals = locals_ref.lock();

    locals.meta_mut().members.push_pair_to_child(
      machine.symbol("stop"), stop_alien_ref);
  }

  ReactorTest {
    init: proc (reactor) {
      // Since it's pristine we can really give it anything we want.
      reactor.stage(caller_ref.clone(), caller_ref);
    },
    fini: proc () {
    }
  }
}

#[test]
fn serial_reactor_stall_handlers() {
  util::timeout(1000, proc() {
    let mut reactor = SerialReactor::new(Machine::new());

    let test = test_reactor_stall_handlers();

    (test.init)(&mut reactor);

    reactor.run();

    (test.fini)();
  })
}

#[test]
fn serial_reactor_react_stop_call() {
  util::timeout(1000, proc() {
    let     machine = Machine::new();
    let mut reactor = SerialReactor::new(machine.clone());

    let test = test_reactor_react_stop_call(&machine);

    (test.init)(&mut reactor);

    reactor.run();

    (test.fini)();
  })
}

static PARALLEL_CONFIGS: [uint, ..3] = [2, 4, 8];

#[test]
fn parallel_reactor_stall_handlers() {
  for &reactors in PARALLEL_CONFIGS.iter() {
    util::timeout(1000, proc() {
      let mut pool = ReactorPool::spawn(Machine::new(), reactors);

      let ReactorTest { init, fini } =
        test_reactor_stall_handlers();

      pool.on_reactor(proc(reactor) {
        init(reactor)
      });

      pool.wait();

      fini();
    })
  }
}

#[test]
fn parallel_reactor_react_stop_call() {
  for &reactors in PARALLEL_CONFIGS.iter() {
    util::timeout(1000, proc() {
      let     machine = Machine::new();
      let mut pool    = ReactorPool::spawn(machine.clone(), reactors);

      let ReactorTest { init, fini } =
        test_reactor_react_stop_call(&machine);

      pool.on_reactor(proc(reactor) {
        init(reactor)
      });

      pool.wait();

      fini();
    })
  }
}
