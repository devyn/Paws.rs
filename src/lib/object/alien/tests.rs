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
    Alien::new(routine, ~~"")
  }

  #[allow(unused_variable)]
  fn routine<'a>(
                 mut alien: TypedRefGuard<'a, Alien>,
                 machine:   &mut Machine,
                 response:  ObjectRef)
                 -> Reaction {

    match response.lock().try_cast::<Symbol>() {
      Ok(symbol) =>
        alien.data.as_mut::<~str>().unwrap()
          .push_str(symbol.deref().name()),
      Err(_) => ()
    }

    Yield
  }
}

#[test]
fn simple_alien() {
  let mut machine = Machine::new();

  let alien_ref = ObjectRef::new(~simple::new_alien());

  let hello = machine.symbol("Hello, ");
  let world = machine.symbol("world!");

  {
    let alien = alien_ref.lock().try_cast::<Alien>().unwrap();
    (alien.routine)(alien, &mut machine, hello);
  }

  {
    let alien = alien_ref.lock().try_cast::<Alien>().unwrap();
    (alien.routine)(alien, &mut machine, world);
  }

  let alien = alien_ref.lock().try_cast::<Alien>().unwrap();

  assert!(alien.deref().data.as_ref::<~str>()
          .unwrap().as_slice() == "Hello, world!");
}
