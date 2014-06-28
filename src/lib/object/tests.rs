use object::*;
use object::empty::Empty;
use object::symbol::Symbol;

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
