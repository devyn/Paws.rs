use system::implementation;

use object::*;
use object::thing::Thing;
use object::alien::Alien;

use machine::*;

#[test]
fn void_accepts_forever() {
  let machine = Machine::new();

  let void   = ObjectRef::new(box implementation::void(&machine));
  let caller = ObjectRef::new(box Thing::new());
  let obj    = ObjectRef::new(box Thing::new());

  let reaction = Alien::realize(
    void.lock().try_cast::<Alien>().ok().unwrap(),
    &machine,
    caller.clone()
  );

  match reaction {
    React(execution, response) => {
      assert!(execution == caller);
      assert!(response  == void);
    },
    _ => fail!("expected React(..)")
  }

  // 100 oughtta be enough to prove 'forever', eh?
  for _ in range(0u, 100) {
    let reaction = Alien::realize(
      void.lock().try_cast::<Alien>().ok().unwrap(),
      &machine,
      obj.clone()
    );

    match reaction {
      React(execution, response) => {
        assert!(execution == caller);
        assert!(response  == void);
      },
      _ => fail!("expected React(..)")
    }
  }
}

#[test]
fn stop_stops() {
  let machine = Machine::new();

  let caller = ObjectRef::new(box Thing::new());

  let reaction = implementation::stop(&machine, caller, &[]);

  assert!(reaction == Yield);
  assert!(machine.dequeue() == None);
}
