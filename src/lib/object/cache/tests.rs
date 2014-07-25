use super::*;

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

  let mut cache = Cache::new();

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

  let mut cache = Cache::new();

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
