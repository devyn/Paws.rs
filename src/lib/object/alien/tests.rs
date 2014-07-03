use machine::*;
use object::*;
use object::alien::Alien;

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
