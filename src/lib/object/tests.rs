use object::*;
use object::empty::Empty;
use object::symbol::Symbol;

use machine::*;

use sync::Arc;

#[test]
fn meta_member_relationships() {
  let object1 = ObjectRef::new(box Empty::new());
  let object2 = ObjectRef::new(box Empty::new());

  let mut meta = Meta::new();

  meta.members.push(object1.clone());

  meta.members.push_child(object2.clone());

  assert!(!meta.members.get(0).unwrap().is_child());
  assert!( meta.members.get(0).unwrap().to() == &object1);

  assert!( meta.members.get(1).unwrap().is_child());
  assert!( meta.members.get(1).unwrap().to() == &object2);
}

#[test]
fn meta_member_push_pair() {
  let key = ObjectRef::new(box Empty::new());
  let val = ObjectRef::new(box Empty::new());

  let mut meta = Meta::new();

  meta.members.push_pair(key.clone(), val.clone());

  meta.members.push_pair_to_child(key.clone(), val.clone());

  assert!(meta.members.len() == 3);

  assert!(meta.members.get(0).is_none());

  assert!(meta.members.get(1).unwrap().is_child());
  assert!(meta.members.get(2).unwrap().is_child());

  {
    // Check non-child pair (1)
    let pair = meta.members.get(1).unwrap().to().lock();
    let pair_members = &pair.deref().meta().members;

    assert!( pair_members.get(0).is_none());

    assert!(!pair_members.get(1).unwrap().is_child());
    assert!( pair_members.get(1).unwrap().to() == &key);

    assert!(!pair_members.get(2).unwrap().is_child()); // should not be child.
    assert!( pair_members.get(2).unwrap().to() == &val);
  }

  {
    // Check child pair (2)
    let pair = meta.members.get(2).unwrap().to().lock();
    let pair_members = &pair.deref().meta().members;

    assert!( pair_members.get(0).is_none());

    assert!(!pair_members.get(1).unwrap().is_child());
    assert!( pair_members.get(1).unwrap().to() == &key);

    assert!( pair_members.get(2).unwrap().is_child()); // should be child.
    assert!( pair_members.get(2).unwrap().to() == &val);
  }
}

#[test]
fn object_ref_guards() {
  let object_ref = ObjectRef::new(box Empty::new());

  assert!(object_ref.lock().meta().members.len() == 0);
}

#[test]
fn typed_ref_guards() {
  let sym        = Arc::new("foo".to_string());
  let object_ref = ObjectRef::new_symbol(box Symbol::new(sym.clone()));

  assert!(object_ref.lock().try_cast::<Empty>().is_err());
  assert!(object_ref.lock().try_cast::<Symbol>().is_ok());

  assert!(object_ref.lock().try_cast::<Symbol>()
            .ok().unwrap().name() == "foo");
}

#[test]
fn symbol_ref_eq_as_symbol() {
  let sym1 = Arc::new("foo".to_string());
  let sym2 = Arc::new("bar".to_string());

  let sym1_ref1 = ObjectRef::new_symbol(box Symbol::new(sym1.clone()));
  let sym1_ref2 = ObjectRef::new_symbol(box Symbol::new(sym1.clone()));

  let sym2_ref1 = ObjectRef::new_symbol(box Symbol::new(sym2.clone()));
  let sym2_ref2 = ObjectRef::new_symbol(box Symbol::new(sym2.clone()));

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
  let empty1_ref = ObjectRef::new(box Empty::new());
  let empty2_ref = ObjectRef::new(box Empty::new());

  // Identity should be false here, because they aren't symbols
  assert!(!empty1_ref.eq_as_symbol(&empty1_ref));
  assert!(!empty2_ref.eq_as_symbol(&empty2_ref));

  // These should all be false too
  assert!(!empty1_ref.eq_as_symbol(&empty2_ref));
  assert!(!empty2_ref.eq_as_symbol(&empty1_ref));
}

#[test]
fn mixed_refs_eq_as_symbol_is_false() {
  let empty_ref  = ObjectRef::new(box Empty::new());
  let symbol_ref = ObjectRef::new(box Symbol::new(
                     Arc::new("foo".to_string())));

  assert!(!empty_ref.eq_as_symbol(&symbol_ref));
  assert!(!symbol_ref.eq_as_symbol(&empty_ref));
}

struct LookupReceiverTestEnv {
  machine:     Machine,

  target_ref:  ObjectRef,
  caller_ref:  ObjectRef,

  obj_key_ref: ObjectRef,
  obj_val_ref: ObjectRef,

  sym_key_sym: Arc<String>,
  sym_val_ref: ObjectRef
}

fn setup_lookup_receiver_test() -> LookupReceiverTestEnv {
  let machine = Machine::new();

  let caller_ref  = ObjectRef::new(box Empty::new());

  let obj_key_ref = ObjectRef::new(box Empty::new());
  let obj_val_ref = ObjectRef::new(box Empty::new());

  let sym_key_ref = machine.symbol("foo");
  let sym_key_sym = sym_key_ref.symbol_ref().unwrap().clone();
  let sym_val_ref = machine.symbol("bar");

  let target_ref = {
    let mut target = box Empty::new();

    target.meta_mut().members.push_pair(
      obj_key_ref.clone(), obj_val_ref.clone());

    target.meta_mut().members.push_pair(
      sym_key_ref.clone(), sym_val_ref.clone());

    ObjectRef::new(target)
  };

  LookupReceiverTestEnv {
    machine:     machine,

    target_ref:  target_ref,
    caller_ref:  caller_ref,

    obj_key_ref: obj_key_ref,
    obj_val_ref: obj_val_ref,

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
    React(ref execution, ref response) => {
      assert!(execution == &env.caller_ref);
      assert!(response  == &env.obj_val_ref);
    },

    _ => fail!("unexpected reaction! expected React(...)")
  }
}

#[test]
fn lookup_receiver_hit_symbol_key() {
  let mut env = setup_lookup_receiver_test();

  let reaction = lookup_receiver(&mut env.machine, Params {
    caller:  env.caller_ref.clone(),
    subject: env.target_ref.clone(),
    message: ObjectRef::new_symbol(box
               Symbol::new(env.sym_key_sym.clone()))
  });

  match reaction {
    React(ref execution, ref response) => {
      assert!(execution == &env.caller_ref);
      assert!(response  == &env.sym_val_ref);
    },

    _ => fail!("unexpected reaction! expected React(...)")
  }
}

#[test]
fn lookup_receiver_miss_object_key() {
  let mut env = setup_lookup_receiver_test();

  let reaction = lookup_receiver(&mut env.machine, Params {
    caller:  env.caller_ref.clone(),
    subject: env.target_ref.clone(),
    message: ObjectRef::new(box Empty::new())
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
