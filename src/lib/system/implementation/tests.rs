use system::implementation;

use object::*;
use object::thing::Thing;
use object::alien::Alien;

use machine::*;
use machine::reactor::MockReactor;

#[test]
fn void_accepts_forever() {
  let     machine = Machine::new();
  let mut reactor = MockReactor::new(machine.clone());

  let void   = ObjectRef::new(box implementation::void(&machine));
  let caller = ObjectRef::new(box Thing::new());
  let obj    = ObjectRef::new(box Thing::new());

  Alien::realize(
    void.lock().try_cast::<Alien>().ok().unwrap(),
    &mut reactor,
    caller.clone()
  );

  match reactor.stagings.shift() {
    Some((execution, response)) => {
      assert!(execution == caller);
      assert!(response  == void);
    },
    None => fail!("stage() wasn't called")
  }

  // 100 oughtta be enough to prove 'forever', eh?
  for _ in range(0u, 100) {
    let reaction = Alien::realize(
      void.lock().try_cast::<Alien>().ok().unwrap(),
      &mut reactor,
      obj.clone()
    );

    match reactor.stagings.shift() {
      Some((execution, response)) => {
        assert!(execution == caller);
        assert!(response  == void);
      },
      None => fail!("stage() wasn't called")
    }
  }
}

#[test]
fn stop_stops() {
  let     machine = Machine::new();
  let mut reactor = MockReactor::new(machine.clone());

  let caller = ObjectRef::new(box Thing::new());

  let reaction = implementation::stop(&mut reactor, caller);

  assert!(reactor.stagings.is_empty());
  assert!(reactor.alive == false);
}
