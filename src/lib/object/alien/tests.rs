use machine::*;
use object::*;
use object::alien::Alien;
use object::empty::Empty;

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
    (alien.routine)(alien, &machine, hello);
  }

  {
    let alien = alien_ref.lock().try_cast::<Alien>()
                  .ok().expect("alien is not an Alien!");
    (alien.routine)(alien, &machine, world);
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

  let caller_ref = ObjectRef::new(box Empty::new());

  let alien_ref = ObjectRef::new(box
                    Alien::new_call_pattern(routine, 3));

  let assert_caller_and_alien = |send| {
    let alien = alien_ref.lock().try_cast::<Alien>().ok().unwrap();

    match (alien.routine)(alien, &machine, send) {
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

    match (alien.routine)(alien, &machine, machine.symbol("c")) {
      React(execution, response) => {
        assert!(&execution == &caller_ref);
        assert!( response.symbol_ref().unwrap().as_slice() == "abc");
      },
      _ => fail!("Unexpected reaction!")
    }
  }
}
