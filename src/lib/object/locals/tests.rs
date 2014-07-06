use object::*;
use object::thing::Thing;

use machine::*;

use object::locals::*;

#[test]
fn locals_receiver_returns_self_when_name_matches() {
  let machine = Machine::new();

  let name   = machine.symbol("locals");
  let locals = ObjectRef::new(box Locals::new(name));
  let caller = ObjectRef::new(box Thing::new());

  let reaction = locals_receiver(&machine, Params {
    caller:  caller.clone(),
    subject: locals.clone(),
    message: machine.symbol("locals")
  });

  assert!(reaction == React(caller, locals));
}

#[test]
fn locals_receiver_can_lookup_members_too() {
  let machine = Machine::new();

  let name   = machine.symbol("locals");
  let locals = ObjectRef::new(box Locals::new(name));
  let caller = ObjectRef::new(box Thing::new());

  let key    = machine.symbol("key");
  let value  = ObjectRef::new(box Thing::new());

  {
    let mut locals = locals.lock();
    locals.meta_mut().members.push_pair_to_child(key.clone(), value.clone());
  }

  let reaction = locals_receiver(&machine, Params {
    caller:  caller.clone(),
    subject: locals.clone(),
    message: key.clone()
  });

  assert!(reaction == React(caller.clone(), value));

  let reaction = locals_receiver(&machine, Params {
    caller:  caller.clone(),
    subject: locals.clone(),
    message: machine.symbol("foo")
  });

  assert!(reaction == Yield);
}
