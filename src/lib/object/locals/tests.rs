use object::*;
use object::thing::Thing;

use machine::*;
use machine::reactor::MockReactor;

use object::locals::*;

#[test]
fn locals_receiver_returns_self_when_name_matches() {
  let     machine = Machine::new();
  let mut reactor = MockReactor::new(machine.clone());

  let name   = machine.symbol("locals");
  let locals = ObjectRef::new(box Locals::new(name));
  let caller = ObjectRef::new(box Thing::new());

  locals_receiver(&mut reactor, Params {
    caller:  caller.clone(),
    subject: locals.clone(),
    message: machine.symbol("locals")
  });

  assert!(reactor.stagings.shift() == Some((caller, locals)));
}

#[test]
fn locals_receiver_can_lookup_members_too() {
  let     machine = Machine::new();
  let mut reactor = MockReactor::new(machine.clone());

  let name   = machine.symbol("locals");
  let locals = ObjectRef::new(box Locals::new(name));
  let caller = ObjectRef::new(box Thing::new());

  let key    = machine.symbol("key");
  let value  = ObjectRef::new(box Thing::new());

  {
    let mut locals = locals.lock();
    locals.meta_mut().members.push_pair_to_child(key.clone(), value.clone());
  }

  locals_receiver(&mut reactor, Params {
    caller:  caller.clone(),
    subject: locals.clone(),
    message: key.clone()
  });

  assert!(reactor.stagings.shift() == Some((caller.clone(), value)));

  locals_receiver(&mut reactor, Params {
    caller:  caller.clone(),
    subject: locals.clone(),
    message: machine.symbol("foo")
  });

  assert!(reactor.stagings.is_empty());
}
