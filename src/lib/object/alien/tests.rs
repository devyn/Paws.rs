use machine::*;
use object::*;
use object::alien::Alien;
use object::thing::Thing;

use std::any::*;

/// Alien that concatenates symbols it receives into its internal data.
mod simple {
  use machine::*;
  use object::*;
  use object::alien::Alien;
  use object::symbol::Symbol;

  use std::any::*;

  pub fn new_alien() -> Alien {
    Alien::new(routine, box String::new())
  }

  #[allow(unused_variable)]
  fn routine<'a>(
             mut alien: TypedRefGuard<'a, Alien>,
             machine:   &Machine,
             response:  ObjectRef)
             -> Reaction {

    match response.lock().try_cast::<Symbol>() {
      Ok(symbol) =>
        alien.data.as_mut::<String>().unwrap()
          .push_str(symbol.deref().name().as_slice()),
      Err(_) => ()
    }

    Yield
  }
}

#[test]
fn simple_alien() {
  let machine = Machine::new();

  let alien_ref = ObjectRef::new(box simple::new_alien());

  let hello = machine.symbol("Hello, ");
  let world = machine.symbol("world!");

  {
    let alien = alien_ref.lock().try_cast::<Alien>()
                  .ok().expect("alien is not an Alien!");
    Alien::realize(alien, &machine, hello);
  }

  {
    let alien = alien_ref.lock().try_cast::<Alien>()
                  .ok().expect("alien is not an Alien!");
    Alien::realize(alien, &machine, world);
  }

  let alien = alien_ref.lock().try_cast::<Alien>()
                .ok().expect("alien is not an Alien!");

  assert!(alien.deref().data.as_ref::<String>()
          .unwrap().as_slice() == "Hello, world!");
}

#[test]
fn call_pattern_alien() {
  let machine = Machine::new();

  // Returns concatenation of arguments if all three are symbols, otherwise
  // fails. (normally you'd just not want to return)
  fn routine<'a>(
             machine: &Machine,
             caller:  ObjectRef,
             args:    &[ObjectRef])
             -> Reaction {

    let cat_str = args.iter().fold(String::new(), |s, o|
      s.append(o.symbol_ref().expect("expected Symbol")
                                            .as_slice()));

    React(caller, machine.symbol(cat_str.as_slice()))
  }

  let caller_ref = ObjectRef::new(box Thing::new());

  let alien_ref = ObjectRef::new(box
                    Alien::new_call_pattern(routine, 3));

  let assert_caller_and_alien = |send| {
    let alien = alien_ref.lock().try_cast::<Alien>().ok().unwrap();

    match Alien::realize(alien, &machine, send) {
      React(execution, response) => {
        assert!(&execution == &caller_ref);
        assert!(&response  == &alien_ref);
      },
      _ => fail!("Unexpected reaction!")
    }
  };

  assert_caller_and_alien(caller_ref.clone());
  assert_caller_and_alien(machine.symbol("a"));
  assert_caller_and_alien(machine.symbol("b"));

  {
    let alien = alien_ref.lock().try_cast::<Alien>().ok().unwrap();

    match Alien::realize(alien, &machine, machine.symbol("c")) {
      React(execution, response) => {
        assert!(&execution == &caller_ref);
        assert!( response.symbol_ref().unwrap().as_slice() == "abc");
      },
      _ => fail!("Unexpected reaction!")
    }
  }
}

#[test]
fn oneshot_alien() {
  let machine = Machine::new();

  fn routine<'a>(machine: &Machine, response: ObjectRef) -> Reaction {
    React(response, machine.symbol("foo"))
  }

  let caller_ref = ObjectRef::new(box Thing::new());

  let alien_ref = ObjectRef::new(box
                    Alien::new_oneshot(routine));

  {
    let alien = alien_ref.lock().try_cast::<Alien>().ok().unwrap();

    match Alien::realize(alien, &machine, caller_ref.clone()) {
      React(execution, response) => {
        assert!(&execution == &caller_ref);
        assert!(response.eq_as_symbol(&machine.symbol("foo")));
      },

      _ => fail!("Unexpected reaction!")
    }
  }

  {
    let alien = alien_ref.lock().try_cast::<Alien>().ok().unwrap();

    match Alien::realize(alien, &machine, caller_ref.clone()) {
      Yield => (),

      _ => fail!("oneshot responded after first invocation")
    }
  }
}

#[test]
fn alien_from_native_receiver() {
  let machine = Machine::new();

  let caller  = ObjectRef::new(box Thing::new());
  let subject = ObjectRef::new(box Thing::new());
  let message = ObjectRef::new(box Thing::new());

  let params  = ObjectRef::new(box Thing::from_meta({

    let mut meta = Meta::new();

    meta.members.push(caller.clone());
    meta.members.push(subject.clone());
    meta.members.push(message.clone());

    meta
  }));

  fn receiver(machine: &Machine, params: Params) -> Reaction {
    machine.enqueue(params.caller.clone(),  params.message.clone());
              React(params.subject.clone(), params.message.clone())
  }

  let alien_ref = ObjectRef::new(box
                    Alien::from_native_receiver(receiver));

  for _ in range(0u, 3) {
    let alien = alien_ref.lock().try_cast::<Alien>().ok().unwrap();

    match Alien::realize(alien, &machine, params.clone()) {
      React(execution, response) => {
        assert!(&execution == &subject);
        assert!(&response  == &message);

        match machine.dequeue() {
          Some(work) => {
            let Realization(ref execution, ref response) = *work;
            assert!(execution == &caller);
            assert!(response  == &message);
          }

          None => fail!("Nothing on the queue!")
        }
      },

      _ => fail!("Unexpected reaction!")
    }
  }
}
