use object::*;
use object::empty::Empty;
use object::symbol::Symbol;

use machine::*;

use sync::Arc;

#[test]
fn meta_member_relationships() {
  let object1 = ObjectRef::new(~Empty::new());
  let object2 = ObjectRef::new(~Empty::new());

  let mut target = Empty::new();

  target.meta_mut().members.push(
    Some(Relationship::new(object1.clone())));

  target.meta_mut().members.push(
    Some(Relationship::new_child(object2.clone())));

  let mut iter = target.meta().members.iter();

  let relationship = iter.next().unwrap().get_ref();
  assert!(!relationship.is_child());
  assert!( relationship.to() == &object1);

  let relationship = iter.next().unwrap().get_ref();
  assert!( relationship.is_child());
  assert!( relationship.to() == &object2);
}

#[test]
fn object_ref_guards() {
  let object_ref = ObjectRef::new(~Empty::new());

  assert!(object_ref.lock().meta().members.len() == 0);
}

#[test]
fn typed_ref_guards() {
  let sym        = Arc::new(~"foo");
  let object_ref = ObjectRef::new_symbol(~Symbol::new(sym.clone()));

  assert!(object_ref.lock().try_cast::<Empty>().is_err());
  assert!(object_ref.lock().try_cast::<Symbol>().unwrap().name() == "foo");
}

#[test]
fn symbol_ref_eq_as_symbol() {
  let sym1 = Arc::new(~"foo");
  let sym2 = Arc::new(~"bar");

  let sym1_ref1 = ObjectRef::new_symbol(~Symbol::new(sym1.clone()));
  let sym1_ref2 = ObjectRef::new_symbol(~Symbol::new(sym1.clone()));

  let sym2_ref1 = ObjectRef::new_symbol(~Symbol::new(sym2.clone()));
  let sym2_ref2 = ObjectRef::new_symbol(~Symbol::new(sym2.clone()));

  // Identity
  assert!( sym1_ref1.eq_as_symbol(&sym1_ref1));
  assert!( sym1_ref2.eq_as_symbol(&sym1_ref2));
  assert!( sym2_ref1.eq_as_symbol(&sym2_ref1));
  assert!( sym2_ref2.eq_as_symbol(&sym2_ref2));

  // True comparisons (both directions)
  assert!( sym1_ref1.eq_as_symbol(&sym1_ref2));
  assert!( sym1_ref2.eq_as_symbol(&sym1_ref1));

  assert!( sym2_ref1.eq_as_symbol(&sym2_ref2));
  assert!( sym2_ref2.eq_as_symbol(&sym2_ref1));

  // False comparisons (both directions)
  assert!(!sym1_ref1.eq_as_symbol(&sym2_ref1));
  assert!(!sym1_ref1.eq_as_symbol(&sym2_ref2));
  assert!(!sym1_ref2.eq_as_symbol(&sym2_ref1));
  assert!(!sym1_ref2.eq_as_symbol(&sym2_ref2));

  assert!(!sym2_ref1.eq_as_symbol(&sym1_ref1));
  assert!(!sym2_ref1.eq_as_symbol(&sym1_ref2));
  assert!(!sym2_ref2.eq_as_symbol(&sym1_ref1));
  assert!(!sym2_ref2.eq_as_symbol(&sym1_ref2));
}

#[test]
fn non_symbol_ref_eq_as_symbol_is_false() {
  let empty1_ref = ObjectRef::new(~Empty::new());
  let empty2_ref = ObjectRef::new(~Empty::new());

  // Identity should be false here, because they aren't symbols
  assert!(!empty1_ref.eq_as_symbol(&empty1_ref));
  assert!(!empty2_ref.eq_as_symbol(&empty2_ref));

  // These should all be false too
  assert!(!empty1_ref.eq_as_symbol(&empty2_ref));
  assert!(!empty2_ref.eq_as_symbol(&empty1_ref));
}

#[test]
fn mixed_refs_eq_as_symbol_is_false() {
  let empty_ref  = ObjectRef::new(~Empty::new());
  let symbol_ref = ObjectRef::new(~Symbol::new(Arc::new(~"foo")));

  assert!(!empty_ref.eq_as_symbol(&symbol_ref));
  assert!(!symbol_ref.eq_as_symbol(&empty_ref));
}

struct LookupReceiverTestEnv {
  machine:     Machine,

  target_ref:  ObjectRef,
  caller_ref:  ObjectRef,

  obj_key_ref: ObjectRef,
  obj_val_ref: ObjectRef,

  sym_key_sym: Arc<~str>,
  sym_key_ref: ObjectRef,
  sym_val_ref: ObjectRef
}

fn make_pair(key: ObjectRef, value: ObjectRef) -> ObjectRef {
  ObjectRef::new(~Empty::new_pair(key, value))
}

fn setup_lookup_receiver_test() -> LookupReceiverTestEnv {
  let mut machine = Machine::new();

  let target_ref  = ObjectRef::new(~Empty::new());
  let caller_ref  = ObjectRef::new(~Empty::new());

  let obj_key_ref = ObjectRef::new(~Empty::new());
  let obj_val_ref = ObjectRef::new(~Empty::new());

  let sym_key_ref = machine.symbol("foo");
  let sym_key_sym = sym_key_ref.symbol_ref().unwrap().clone();
  let sym_val_ref = machine.symbol("bar");

  {
    let mut target = target_ref.lock().try_cast::<Empty>().unwrap();

    target.meta_mut().members.push(None);

    target.meta_mut().members.push(Some(Relationship::new(
      make_pair(obj_key_ref.clone(), obj_val_ref.clone()))));

    target.meta_mut().members.push(Some(Relationship::new(
      make_pair(sym_key_ref.clone(), sym_val_ref.clone()))));
  }

  LookupReceiverTestEnv {
    machine:     machine,

    target_ref:  target_ref,
    caller_ref:  caller_ref,

    obj_key_ref: obj_key_ref,
    obj_val_ref: obj_val_ref,

    sym_key_ref: sym_key_ref,
    sym_key_sym: sym_key_sym,
    sym_val_ref: sym_val_ref
  }
}

#[test]
fn lookup_receiver_hit_object_key() {
  let mut env = setup_lookup_receiver_test();

  let reaction = lookup_receiver(&mut env.machine, Params {
    caller:  env.caller_ref.clone(),
    subject: env.target_ref.clone(),
    message: env.obj_key_ref.clone()
  });

  match reaction {
    React(Stage(ref stage_params)) => {
      assert!(&stage_params.execution == &env.caller_ref);
      assert!(&stage_params.response  == &env.obj_val_ref);
    },

    _ => fail!("unexpected reaction!")
  }
}

#[test]
fn lookup_receiver_hit_symbol_key() {
  let mut env = setup_lookup_receiver_test();

  let reaction = lookup_receiver(&mut env.machine, Params {
    caller:  env.caller_ref.clone(),
    subject: env.target_ref.clone(),
    message: ObjectRef::new_symbol(
               ~Symbol::new(env.sym_key_sym.clone()))
  });

  match reaction {
    React(Stage(ref stage_params)) => {
      assert!(&stage_params.execution == &env.caller_ref);
      assert!(&stage_params.response  == &env.sym_val_ref);
    },

    _ => fail!("unexpected reaction! expected React(Stage(...))")
  }
}

#[test]
fn lookup_receiver_miss_object_key() {
  let mut env = setup_lookup_receiver_test();

  let reaction = lookup_receiver(&mut env.machine, Params {
    caller:  env.caller_ref.clone(),
    subject: env.target_ref.clone(),
    message: ObjectRef::new(~Empty::new())
  });

  match reaction {
    Yield => (),

    _ => fail!("unexpected reaction! expected Yield")
  }
}

#[test]
fn lookup_receiver_miss_symbol_key() {
  let mut env = setup_lookup_receiver_test();

  let bar_sym_ref = env.machine.symbol("bar");

  let reaction = lookup_receiver(&mut env.machine, Params {
    caller:  env.caller_ref.clone(),
    subject: env.target_ref.clone(),
    message: bar_sym_ref
  });

  match reaction {
    Yield => (),

    _ => fail!("unexpected reaction! expected Yield")
  }
}
