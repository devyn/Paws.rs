use super::Cache;

use object;

use machine::Machine;

use nuketype::Thing;

#[test]
pub fn sym_lookup_miss_and_hit() {
  let machine = Machine::new();

  let foo_sym = machine.symbol_map.lock().intern("foo");
  let bar_sym = machine.symbol_map.lock().intern("bar");

  let foo = Thing::empty();
  let bar = Thing::empty();

  let dictionary = Thing::from_fn(|dictionary| {
    dictionary.members.push_pair(machine.symbol("foo"), foo.clone());
    dictionary.members.push_pair(machine.symbol("bar"), bar.clone());
  });

  let mut cache = Cache::new_serial();

  assert_eq!(0, cache.stats().sym_lookup_misses);
  assert_eq!(0, cache.stats().sym_lookup_hits);

  // foo: expect miss
  let result = cache.sym_lookup(dictionary.clone(), foo_sym.clone())
                 .expect("sym_lookup(foo_sym) returned None!");

  assert_eq!(foo, result);

  assert_eq!(1, cache.stats().sym_lookup_misses);
  assert_eq!(0, cache.stats().sym_lookup_hits);

  // foo: expect hit
  let result = cache.sym_lookup(dictionary.clone(), foo_sym.clone())
                 .expect("sym_lookup(foo_sym) returned None!");

  assert_eq!(foo, result);

  assert_eq!(1, cache.stats().sym_lookup_misses);
  assert_eq!(1, cache.stats().sym_lookup_hits);

  // bar: expect miss
  let result = cache.sym_lookup(dictionary.clone(), bar_sym.clone())
                 .expect("sym_lookup(bar_sym) returned None!");

  assert_eq!(bar, result);

  assert_eq!(2, cache.stats().sym_lookup_misses);
  assert_eq!(1, cache.stats().sym_lookup_hits);

  // bar: expect hit
  let result = cache.sym_lookup(dictionary.clone(), bar_sym.clone())
                 .expect("sym_lookup(bar_sym) returned None!");

  assert_eq!(bar, result);

  assert_eq!(2, cache.stats().sym_lookup_misses);
  assert_eq!(2, cache.stats().sym_lookup_hits);
}

#[test]
pub fn sym_lookup_invalidate() {
  let machine = Machine::new();

  let foo_sym = machine.symbol_map.lock().intern("foo");

  let foo1 = Thing::empty();
  let foo2 = Thing::empty();

  let pair = Thing::pair(machine.symbol("foo"), foo1.clone());

  let dictionary = Thing::from_fn(|dictionary| {
    dictionary.members.push(pair.clone());
  });

  let mut cache = Cache::new_serial();

  // Prime the cache by allowing it to see `foo`.
  cache.sym_lookup(dictionary.clone(), foo_sym.clone());

  // Ensure we have a hit after priming it.
  let result = cache.sym_lookup(dictionary.clone(), foo_sym.clone()).unwrap();

  assert_eq!(foo1, result);
  assert_eq!(1, cache.stats().sym_lookup_hits);

  // Now invalidate by changing the pair to point to foo2 instead.
  pair.lock().meta_mut().members.set(2, foo2.clone());

  // And verify that the cache has, indeed, noticed this:
  let result = cache.sym_lookup(dictionary.clone(), foo_sym.clone()).unwrap();

  assert_eq!(foo2, result);
  assert_eq!(2, cache.stats().sym_lookup_misses);

  // Make sure it hits again after, though.
  let result = cache.sym_lookup(dictionary.clone(), foo_sym.clone()).unwrap();

  assert_eq!(foo2, result);
  assert_eq!(2, cache.stats().sym_lookup_hits);

  // Now invalidate again by changing the dictionary to remove `foo` altogether!
  dictionary.lock().meta_mut().members.remove(1);

  // Verify again:
  let result = cache.sym_lookup(dictionary.clone(), foo_sym.clone());

  assert_eq!(None, result);
  assert_eq!(3, cache.stats().sym_lookup_misses);
}

#[test]
pub fn receiver_miss_and_hit() {
  let receiver1 = Thing::empty();
  let receiver2 = Thing::empty();

  let object1 = Thing::from_fn(|meta| {
    meta.receiver = object::ObjectReceiver(receiver1.clone());
  });

  let object2 = Thing::from_fn(|meta| {
    meta.receiver = object::ObjectReceiver(receiver2.clone());
  });

  // Ensure the versions are > 0
  object1.lock().meta_mut();
  object2.lock().meta_mut();

  let mut cache = Cache::new_parallel();

  assert_eq!(0, cache.stats().receiver_misses);
  assert_eq!(0, cache.stats().receiver_hits);

  // object1 :: receiver1: miss
  match cache.receiver(object1.clone()) {
    object::ObjectReceiver(receiver) =>
      assert_eq!(receiver1, receiver),

    _ =>
      fail!("expected ObjectReceiver")
  }

  assert_eq!(1, cache.stats().receiver_misses);
  assert_eq!(0, cache.stats().receiver_hits);

  // object1 :: receiver1: hit
  match cache.receiver(object1.clone()) {
    object::ObjectReceiver(receiver) =>
      assert_eq!(receiver1, receiver),

    _ =>
      fail!("expected ObjectReceiver")
  }

  assert_eq!(1, cache.stats().receiver_misses);
  assert_eq!(1, cache.stats().receiver_hits);

  // object2 :: receiver2: miss
  match cache.receiver(object2.clone()) {
    object::ObjectReceiver(receiver) =>
      assert_eq!(receiver2, receiver),

    _ =>
      fail!("expected ObjectReceiver")
  }

  assert_eq!(2, cache.stats().receiver_misses);
  assert_eq!(1, cache.stats().receiver_hits);

  // object2 :: receiver2: hit
  match cache.receiver(object2.clone()) {
    object::ObjectReceiver(receiver) =>
      assert_eq!(receiver2, receiver),

    _ =>
      fail!("expected ObjectReceiver")
  }

  assert_eq!(2, cache.stats().receiver_misses);
  assert_eq!(2, cache.stats().receiver_hits);
}

#[test]
pub fn receiver_invalidate() {
  let receiver1 = Thing::empty();
  let receiver2 = Thing::empty();

  let object = Thing::from_fn(|meta| {
    meta.receiver = object::ObjectReceiver(receiver1.clone());
  });

  // Ensure the version is > 0
  object.lock().meta_mut();

  let mut cache = Cache::new_parallel();

  // Prime the cache
  cache.receiver(object.clone());

  assert_eq!(1, cache.stats().receiver_misses);
  assert_eq!(0, cache.stats().receiver_hits);

  // Make sure it is indeed returning receiver1
  match cache.receiver(object.clone()) {
    object::ObjectReceiver(receiver) =>
      assert_eq!(receiver1, receiver),

    _ =>
      fail!("expected ObjectReceiver")
  }

  assert_eq!(1, cache.stats().receiver_misses);
  assert_eq!(1, cache.stats().receiver_hits);

  // Now invalidate the cache by setting the receiver to receiver2
  object.lock().meta_mut().receiver =
    object::ObjectReceiver(receiver2.clone());

  // Make sure it immediately returns receiver2
  match cache.receiver(object.clone()) {
    object::ObjectReceiver(receiver) =>
      assert_eq!(receiver2, receiver),

    _ =>
      fail!("expected ObjectReceiver")
  }

  assert_eq!(2, cache.stats().receiver_misses);
  assert_eq!(1, cache.stats().receiver_hits);

  // And make sure it can still hit after changing it
  cache.receiver(object.clone());

  assert_eq!(2, cache.stats().receiver_misses);
  assert_eq!(2, cache.stats().receiver_hits);
}
