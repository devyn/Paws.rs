use super::*;

use object::{ObjectRef, Params, Meta};

use nuketype::Thing;

use machine::Machine;
use machine::reactor::{Reactor, MockReactor};

use std::any::AnyRefExt;

/// Alien that concatenates symbols it receives into its internal data.
mod simple {
  use object::{ObjectRef, TypedRefGuard};

  use nuketype::{Alien, Symbol};

  use machine::Reactor;

  use std::any::AnyMutRefExt;

  pub fn create() -> ObjectRef {
    Alien::create("simple", routine, box String::new())
  }

  #[allow(unused_variable)]
  fn routine<'a>(
             mut alien: TypedRefGuard<'a, Alien>,
             reactor:   &mut Reactor,
             response:  ObjectRef) {

    match response.lock().try_cast::<Symbol>() {
      Ok(symbol) =>
        alien.data.as_mut::<String>().unwrap()
          .push_str(symbol.deref().name().as_slice()),
      Err(_) => ()
    }
  }
}

#[test]
fn simple_alien() {
  let     machine = Machine::new();
  let mut reactor = MockReactor::new(machine.clone());

  let alien_ref = simple::create();

  let hello = machine.symbol("Hello, ");
  let world = machine.symbol("world!");

  {
    let alien = alien_ref.lock().try_cast::<Alien>()
                  .ok().expect("alien is not an Alien!");
    Alien::realize(alien, &mut reactor, hello);
  }

  {
    let alien = alien_ref.lock().try_cast::<Alien>()
                  .ok().expect("alien is not an Alien!");
    Alien::realize(alien, &mut reactor, world);
  }

  let alien = alien_ref.lock().try_cast::<Alien>()
                .ok().expect("alien is not an Alien!");

  assert!(alien.deref().data.as_ref::<String>()
          .unwrap().as_slice() == "Hello, world!");
}

#[test]
fn call_pattern_alien() {
  let     machine = Machine::new();
  let mut reactor = MockReactor::new(machine.clone());

  // Returns concatenation of arguments if all three are symbols, otherwise
  // fails. (normally you'd just not want to return)
  fn routine(reactor: &mut Reactor, caller: ObjectRef, args: &[ObjectRef]) {

    let cat_str = args.iter().fold(String::new(), |s, o|
      s.append(o.symbol_ref().expect("expected Symbol")
                                            .as_slice()));

    let symbol = reactor.machine().symbol(cat_str.as_slice());

    reactor.stage(caller, symbol);
  }

  let caller_ref = Thing::empty();

  let alien_ref = Alien::call_pattern("routine", routine, 3);

  let assert_caller_and_alien = |reactor: &mut MockReactor, send| {
    let alien = alien_ref.lock().try_cast::<Alien>().ok().unwrap();

    Alien::realize(alien, reactor, send);

    match reactor.stagings.as_slice() {
      [(ref execution, ref response)] => {
        assert!(execution == &caller_ref);
        assert!(response  == &alien_ref);
      },
      _ => fail!("Unexpected reaction!")
    }

    reactor.stagings.truncate(0);
  };

  assert_caller_and_alien(&mut reactor, caller_ref.clone());
  assert_caller_and_alien(&mut reactor, machine.symbol("a"));
  assert_caller_and_alien(&mut reactor, machine.symbol("b"));

  {
    let alien = alien_ref.lock().try_cast::<Alien>().ok().unwrap();

    Alien::realize(alien, &mut reactor, machine.symbol("c"));

    match reactor.stagings.as_slice() {
      [(ref execution, ref response)] => {
        assert!(execution == &caller_ref);
        assert!(response.symbol_ref().unwrap().as_slice() == "abc");
      },
      _ => fail!("Unexpected reaction!")
    }

    reactor.stagings.truncate(0);
  }

  {
    let alien = alien_ref.lock().try_cast::<Alien>().ok().unwrap();

    Alien::realize(alien, &mut reactor, machine.symbol("d"));

    assert!(reactor.stagings.is_empty()); // already complete
  }
}

#[test]
fn oneshot_alien() {
  let     machine = Machine::new();
  let mut reactor = MockReactor::new(machine.clone());

  fn routine(reactor: &mut Reactor, response: ObjectRef) {
    let symbol = reactor.machine().symbol("foo");

    reactor.stage(response, symbol);
  }

  let caller_ref = Thing::empty();

  let alien_ref = Alien::oneshot("routine", routine);

  {
    let alien = alien_ref.lock().try_cast::<Alien>().ok().unwrap();

    Alien::realize(alien, &mut reactor, caller_ref.clone());
    
    match reactor.stagings.as_slice() {
      [(ref execution, ref response)] => {
        assert!(execution == &caller_ref);
        assert!(response.eq_as_symbol(&machine.symbol("foo")));
      },

      _ => fail!("Unexpected reaction!")
    }

    reactor.stagings.truncate(0);
  }

  {
    let alien = alien_ref.lock().try_cast::<Alien>().ok().unwrap();

    Alien::realize(alien, &mut reactor, caller_ref.clone());

    assert!(reactor.stagings.is_empty());
  }
}

#[test]
fn alien_from_native_receiver() {
  let     machine = Machine::new();
  let mut reactor = MockReactor::new(machine.clone());

  let caller  = Thing::empty();
  let subject = Thing::empty();
  let message = Thing::empty();

  let params  = Thing::create({

    let mut meta = Meta::new();

    meta.members.push(caller.clone());
    meta.members.push(subject.clone());
    meta.members.push(message.clone());

    meta
  });

  fn receiver(reactor: &mut Reactor, params: Params) {
    reactor.stage(params.caller.clone(),  params.message.clone());
    reactor.stage(params.subject.clone(), params.message.clone());
  }

  let alien_ref = Alien::from_native_receiver(receiver);

  for _ in range(0u, 3) {
    let alien = alien_ref.lock().try_cast::<Alien>().ok().unwrap();

    Alien::realize(alien, &mut reactor, params.clone());

    match reactor.stagings.as_slice() {
      [(ref execution1, ref response1), (ref execution2, ref response2)] => {
        assert!(execution1 == &caller);
        assert!(response1  == &message);

        assert!(execution2 == &subject);
        assert!(response2  == &message);
      },

      _ => fail!("Unexpected reaction!")
    }

    reactor.stagings.truncate(0);
  }
}
