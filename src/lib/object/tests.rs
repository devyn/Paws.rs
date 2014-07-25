use super::*;

use nuketype::{Thing, Symbol};

use machine::Machine;
use machine::reactor::MockReactor;

use std::sync::Arc;

#[test]
fn members_set_and_get() {
  let object1 = Thing::empty();
  let object2 = Thing::empty();

  let mut members = Members::new();

  members.set(2, object2.clone());

  assert!(!members.get(2).unwrap().is_child());
  assert!( members.get(2).unwrap().to() == &object2);

  assert!( members.get(1).is_none());
  assert!( members.get(0).is_none());
  assert!( members.get(3).is_none());

  members.set_child(1, object1.clone());

  assert!( members.get(1).unwrap().is_child());
  assert!( members.get(1).unwrap().to() == &object1);

  assert!( members.get(0).is_none());
  assert!( members.get(3).is_none());

  assert!( members.set_child(2, object1.clone()) ==
             Some(Relationship::new(object2)) );

  assert!( members.get(2).unwrap().to() == &object1);
}

#[test]
fn members_iter() {
  let object0 = Thing::empty();
  let object1 = Thing::empty();
  let object2 = Thing::empty();

  let mut members = Members::new();

  members.vec.push(Some(Relationship::new(object0.clone())));
  members.vec.push(Some(Relationship::new(object1.clone())));
  members.vec.push(Some(Relationship::new(object2.clone())));

  let mut iter = members.iter();

  // Skips index 0 (noughty)
  assert!(iter.next().unwrap() == &Some(Relationship::new(object1)));
  assert!(iter.next().unwrap() == &Some(Relationship::new(object2)));
  assert!(iter.next().is_none());
}

#[test]
fn members_own_and_disown() {
  let object = Thing::empty();

  let mut members = Members::new();

  members.vec.push(Some(Relationship::new(object.clone())));

  assert!(!members.vec[0].get_ref().is_child());

  members.own(0);

  assert!( members.vec[0].get_ref().is_child());

  members.disown(0);

  assert!(!members.vec[0].get_ref().is_child());
}

#[test]
fn members_push() {
  let object1 = Thing::empty();
  let object2 = Thing::empty();

  let mut members = Members::new();

  members.push(      object1.clone());
  members.push_child(object2.clone());

  assert!(!members.vec[1].get_ref().is_child());
  assert!( members.vec[1].get_ref().to() == &object1);

  assert!( members.vec[2].get_ref().is_child());
  assert!( members.vec[2].get_ref().to() == &object2);
}

#[test]
fn members_pop() {
  let object0 = Thing::empty();
  let object1 = Thing::empty();
  let object2 = Thing::empty();

  let mut members = Members::new();

  members.vec.push(Some(Relationship::new(object0.clone())));
  members.vec.push(Some(Relationship::new(object1.clone())));
  members.vec.push(Some(Relationship::new(object2.clone())));

  assert!(members.pop() == Some(Relationship::new(object2)));
  assert!(members.pop() == Some(Relationship::new(object1)));
  assert!(members.pop().is_none());

  assert!(members.vec.pop() == Some(Some(Relationship::new(object0))));
}

#[test]
fn members_insert() {
  let object1 = Thing::empty();
  let object2 = Thing::empty();

  let mut members = Members::new();

  members.insert(2, object2.clone());

  assert!(!members.get(2).unwrap().is_child());
  assert!( members.get(2).unwrap().to() == &object2);

  assert!( members.get(1).is_none());
  assert!( members.get(0).is_none());
  assert!( members.get(3).is_none());

  members.insert_child(1, object1.clone());

  assert!( members.get(1).unwrap().is_child());
  assert!( members.get(1).unwrap().to() == &object1);

  assert!(!members.get(3).unwrap().is_child());
  assert!( members.get(3).unwrap().to() == &object2);

  assert!( members.get(0).is_none());
  assert!( members.get(2).is_none());
}

#[test]
fn members_remove() {
  let object0 = Thing::empty();
  let object1 = Thing::empty();
  let object2 = Thing::empty();

  let mut members = Members::new();

  members.vec.push(Some(Relationship::new(object0.clone())));
  members.vec.push(Some(Relationship::new(object1.clone())));
  members.vec.push(Some(Relationship::new(object2.clone())));

  assert!(members.remove(2) == Some(Relationship::new(object2)));
  assert!(members.vec.len() == 2);
  
  assert!(members.remove(0) == Some(Relationship::new(object0)));
  assert!(members.vec.len() == 1);

  assert!(members.remove(1).is_none());
  assert!(members.remove(0) == Some(Relationship::new(object1)));

  assert!(members.vec.is_empty());
}

#[test]
fn members_delete() {
  let object0 = Thing::empty();

  let mut members = Members::new();

  members.vec.push(Some(Relationship::new(object0.clone())));

  assert!(members.delete(1).is_none());

  assert!(members.delete(0) == Some(Relationship::new(object0)));

  assert!(members.vec.len() == 1);
  assert!(members.vec[0].is_none());
}

#[test]
fn members_lookup_pair_by_ref_equality() {
  let key1 = Thing::empty();
  let key2 = Thing::empty();
  let key3 = Thing::empty(); // doesn't exist

  let val1 = Thing::empty();
  let val2 = Thing::empty();

  let mut members = Members::new();

  members.push_pair(key1.clone(), val1.clone());
  members.push_pair(key2.clone(), val2.clone());

  assert!(members.lookup_pair(&key1) == Some(val1));
  assert!(members.lookup_pair(&key2) == Some(val2));
  assert!(members.lookup_pair(&key3) == None);
}

#[test]
fn members_lookup_pair_by_symbol_equality() {
  let machine = Machine::new();

  let key1 = machine.symbol("key1");
  let key2 = machine.symbol("key2");

  let val1 = Thing::empty();
  let val2 = Thing::empty();

  let mut members = Members::new();

  members.push_pair(key1, val1.clone());
  members.push_pair(key2, val2.clone());

  assert!(members.lookup_pair(&machine.symbol("key1")) == Some(val1));
  assert!(members.lookup_pair(&machine.symbol("key2")) == Some(val2));
  assert!(members.lookup_pair(&machine.symbol("key3")) == None);
}

#[test]
fn members_lookup_pair_on_empty_members() {
  let key     = Thing::empty();
  let members = Members::new();

  assert!(members.lookup_pair(&key) == None);
}

#[test]
fn members_push_pair() {
  let key = Thing::empty();
  let val = Thing::empty();

  let mut members = Members::new();

  members.push_pair(key.clone(), val.clone());

  members.push_pair_to_child(key.clone(), val.clone());

  assert!(members.len() == 3);

  assert!(members.get(0).is_none());

  assert!(members.get(1).unwrap().is_child());
  assert!(members.get(2).unwrap().is_child());

  {
    // Check non-child pair (1)
    let pair = members.get(1).unwrap().to().lock();
    let pair_members = &pair.meta().members;

    assert!( pair_members.get(0).is_none());

    assert!(!pair_members.get(1).unwrap().is_child());
    assert!( pair_members.get(1).unwrap().to() == &key);

    assert!(!pair_members.get(2).unwrap().is_child()); // should not be child.
    assert!( pair_members.get(2).unwrap().to() == &val);
  }

  {
    // Check child pair (2)
    let pair = members.get(2).unwrap().to().lock();
    let pair_members = &pair.meta().members;

    assert!( pair_members.get(0).is_none());

    assert!(!pair_members.get(1).unwrap().is_child());
    assert!( pair_members.get(1).unwrap().to() == &key);

    assert!( pair_members.get(2).unwrap().is_child()); // should be child.
    assert!( pair_members.get(2).unwrap().to() == &val);
  }
}

#[test]
fn members_expand_to() {
  let mut members = Members::new();

  assert!(members.vec.is_empty());

  members.expand_to(1);
  assert!(members.vec.len() == 1);

  members.expand_to(3);
  assert!(members.vec.len() == 3);

  assert!(members.vec[0].is_none());
  assert!(members.vec[1].is_none());
  assert!(members.vec[2].is_none());
}

#[test]
fn members_len() {
  let mut members = Members::new();

  assert!(members.is_empty());

  members.vec.push(None);
  members.vec.push(None);
  members.vec.push(None);

  assert!(members.vec.len() == members.len());
}

#[test]
fn object_ref_equality() {
  let object_ref1 = Thing::empty();
  let object_ref2 = Thing::empty();

  assert!(&object_ref1 == &object_ref1);
  assert!(&object_ref1 != &object_ref2);
}

#[test]
fn object_ref_guards() {
  let object_ref = Thing::empty();

  assert!(object_ref.lock().meta().members.len() == 0);
}

#[test]
fn typed_ref_guards() {
  let sym        = Arc::new("foo".to_string());
  let object_ref = ObjectRef::store_symbol(box Symbol::new(sym.clone()));

  assert!(object_ref.lock().try_cast::<Thing>().is_err());
  assert!(object_ref.lock().try_cast::<Symbol>().is_ok());

  assert!(object_ref.lock().try_cast::<Symbol>()
            .ok().unwrap().name() == "foo");
}

#[test]
fn symbol_ref_eq_as_symbol() {
  let sym1 = Arc::new("foo".to_string());
  let sym2 = Arc::new("bar".to_string());

  let sym1_ref1 = ObjectRef::store_symbol(box Symbol::new(sym1.clone()));
  let sym1_ref2 = ObjectRef::store_symbol(box Symbol::new(sym1.clone()));

  let sym2_ref1 = ObjectRef::store_symbol(box Symbol::new(sym2.clone()));
  let sym2_ref2 = ObjectRef::store_symbol(box Symbol::new(sym2.clone()));

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
  let thing1_ref = Thing::empty();
  let thing2_ref = Thing::empty();

  // Identity should be false here, because they aren't symbols
  assert!(!thing1_ref.eq_as_symbol(&thing1_ref));
  assert!(!thing2_ref.eq_as_symbol(&thing2_ref));

  // These should all be false too
  assert!(!thing1_ref.eq_as_symbol(&thing2_ref));
  assert!(!thing2_ref.eq_as_symbol(&thing1_ref));
}

#[test]
fn mixed_refs_eq_as_symbol_is_false() {
  let thing_ref  = Thing::empty();
  let symbol_ref = ObjectRef::store_symbol(box Symbol::new(
                     Arc::new("foo".to_string())));

  assert!(!thing_ref.eq_as_symbol(&symbol_ref));
  assert!(!symbol_ref.eq_as_symbol(&thing_ref));
}

struct LookupReceiverTestEnv {
  machine:     Machine,
  reactor:     MockReactor,

  target_ref:  ObjectRef,
  caller_ref:  ObjectRef,

  obj_key_ref: ObjectRef,
  obj_val_ref: ObjectRef,

  sym_key_sym: Arc<String>,
  sym_val_ref: ObjectRef
}

fn setup_lookup_receiver_test() -> LookupReceiverTestEnv {
  let machine = Machine::new();
  let reactor = MockReactor::new(machine.clone());

  let caller_ref  = Thing::empty();

  let obj_key_ref = Thing::empty();
  let obj_val_ref = Thing::empty();

  let sym_key_ref = machine.symbol("foo");
  let sym_key_sym = sym_key_ref.symbol_ref().unwrap().clone();
  let sym_val_ref = machine.symbol("bar");

  let target_ref = {
    let mut target = Meta::new();

    target.members.push_pair(
      obj_key_ref.clone(), obj_val_ref.clone());

    target.members.push_pair(
      sym_key_ref.clone(), sym_val_ref.clone());

    Thing::create(target)
  };

  LookupReceiverTestEnv {
    machine:     machine,
    reactor:     reactor,

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

  lookup_receiver(&mut env.reactor, Params {
    caller:  env.caller_ref.clone(),
    subject: env.target_ref.clone(),
    message: env.obj_key_ref.clone()
  });

  match env.reactor.stagings.as_slice() {
    [(ref execution, ref response)] => {
      assert!(execution == &env.caller_ref);
      assert!(response  == &env.obj_val_ref);
    },

    _ => fail!("unexpected reaction!")
  }
}

#[test]
fn lookup_receiver_hit_symbol_key() {
  let mut env = setup_lookup_receiver_test();

  lookup_receiver(&mut env.reactor, Params {
    caller:  env.caller_ref.clone(),
    subject: env.target_ref.clone(),
    message: ObjectRef::store_symbol(box
               Symbol::new(env.sym_key_sym.clone()))
  });

  match env.reactor.stagings.as_slice() {
    [(ref execution, ref response)] => {
      assert!(execution == &env.caller_ref);
      assert!(response  == &env.sym_val_ref);
    },

    _ => fail!("unexpected reaction!")
  }
}

#[test]
fn lookup_receiver_miss_object_key() {
  let mut env = setup_lookup_receiver_test();

  lookup_receiver(&mut env.reactor, Params {
    caller:  env.caller_ref.clone(),
    subject: env.target_ref.clone(),
    message: Thing::empty()
  });

  assert!(env.reactor.stagings.is_empty());
}

#[test]
fn lookup_receiver_miss_symbol_key() {
  let mut env = setup_lookup_receiver_test();

  let bar_sym_ref = env.machine.symbol("bar");

  lookup_receiver(&mut env.reactor, Params {
    caller:  env.caller_ref.clone(),
    subject: env.target_ref.clone(),
    message: bar_sym_ref
  });

  assert!(env.reactor.stagings.is_empty());
}
