use object::*;
use object::empty::Empty;

#[test]
fn meta_member_relationships() {
  let object1 = ObjectRef::new(~Empty::new());
  let object2 = ObjectRef::new(~Empty::new());

  let mut target = Empty::new();

  target.meta_mut().members.push(
    Relationship::new(object1.clone()));

  target.meta_mut().members.push(
    Relationship::new_child(object2.clone()));

  let mut iter = target.meta().members.iter();

  {
    let relationship = iter.next().unwrap();
    assert!(!relationship.is_child());
    assert!( relationship.deref() == &object1);
  }
  {
    let relationship = iter.next().unwrap();
    assert!( relationship.is_child());
    assert!( relationship.deref() == &object2);
  }
}
