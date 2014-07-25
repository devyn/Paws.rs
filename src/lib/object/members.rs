use std::mem::replace;
use std::slice::Items;

use object::{ObjectRef, Relationship};
use nuketype::Thing;

/// A list of Relationships that make up an Object's members.
///
/// The list is allowed to have 'holes', so the Relationships are optional.
///
/// Note that 'nuclear' algorithms (i.e. those part of Paws' Nucleus, which is
/// what Paws.rs strives to implement) should never assume anything about the
/// first index of the list and should instead start from the second index
/// unless specifically requested not to, as per the 'noughty' rule (see spec).
///
/// As such, all of the methods of `Members` that don't explicitly take an
/// index, including `push()`, `unshift()`, etc. **skip index 0 completely**.
///
/// There is one exception: `len()` is the length of the underlying vector, so
/// it still counts `noughty`. You probably want to subtract it.
#[deriving(Clone)]
pub struct Members {
  /// The vector of members, as `Option<Relationship>`s to account for holes.
  /// This is publicly exposed because `Vec` simply defines way too many useful
  /// methods, and there's no point in redefining all of them.
  pub vec: Vec<Option<Relationship>>
}

impl Members {
  /// Construct a new members list.
  pub fn new() -> Members {
    Members { vec: Vec::new() }
  }

  /// Gets a reference to the Relationship at the given index, if there is one.
  pub fn get<'a>(&'a self, index: uint) -> Option<&'a Relationship> {
    if index >= self.len() {
      None
    } else {
      self.vec[index].as_ref()
    }
  }

  /// Returns an iterator over the Relationships and holes in the list.
  ///
  /// Skips over the first element to obey the noughty rule.
  pub fn iter<'a>(&'a self) -> Items<'a, Option<Relationship>> {
    // Have to check, because tail() fails on a zero-element slice.
    if self.len() > 0 {
      self.vec.tail().iter()
    } else {
      self.vec.slice(0, 0).iter()
    }
  }

  /// Replaces the object at the given position with a new non-child
  /// Relationship to the given object.
  ///
  /// Returns the Relationship that was replaced, if one was.
  ///
  /// Holes may be created if the index doesn't exist.
  pub fn set(&mut self, index: uint, object: ObjectRef)
             -> Option<Relationship> {

    if index >= self.len() {
      self.expand_to(index);
      self.push(object);
      None
    } else {
      replace(self.vec.get_mut(index), Some(Relationship::new(object)))
    }
  }

  /// Replaces the object at the given position with a new child Relationship to
  /// the given object.
  ///
  /// Returns the Relationship that was replaced, if one was.
  ///
  /// Holes may be created if the index doesn't exist.
  pub fn set_child(&mut self, index: uint, object: ObjectRef)
             -> Option<Relationship> {

    if index >= self.len() {
      self.expand_to(index);
      self.push_child(object);
      None
    } else {
      replace(self.vec.get_mut(index), Some(Relationship::new_child(object)))
    }
  }

  /// Turns the relationship at the given position into a child Relationship.
  ///
  /// Returns `true` if there was a relationship at the index, `false`
  /// otherwise.
  pub fn own(&mut self, index: uint) -> bool {
    if index < self.len() {
      match *self.vec.get_mut(index) {
        Some(ref mut relationship) => {
          relationship.own();
          true
        },
        None => false
      }
    } else {
      false
    }
  }

  /// Turns the relationship at the given position into a non-child
  /// Relationship.
  ///
  /// Returns `true` if there was a relationship at the index, `false`
  /// otherwise.
  pub fn disown(&mut self, index: uint) -> bool {
    if index < self.len() {
      match *self.vec.get_mut(index) {
        Some(ref mut relationship) => {
          relationship.disown();
          true
        },
        None => false
      }
    } else {
      false
    }
  }

  /// Affixes the given object as a non-child Relationship.
  pub fn push(&mut self, object: ObjectRef) {
    self.expand_to(1);
    self.vec.push(Some(Relationship::new(object)));
  }

  /// Affixes the given object as a child Relationship.
  pub fn push_child(&mut self, object: ObjectRef) {
    self.expand_to(1);
    self.vec.push(Some(Relationship::new_child(object)));
  }

  /// Removes and returns the last Relationship, unless the list is empty or
  /// there was a hole at the end.
  ///
  /// Obeys the noughty rule, so 'empty' is defined as 'one or fewer' elements.
  pub fn pop(&mut self) -> Option<Relationship> {
    if self.len() > 1 {
      self.vec.pop().unwrap()
    } else {
      None
    }
  }

  /// Inserts the given object as a non-child Relationship at the given index,
  /// pushing further Relationships upward.
  ///
  /// Holes may be created if the index doesn't exist.
  pub fn insert(&mut self, index: uint, object: ObjectRef) {
    if index >= self.len() {
      self.expand_to(index);
      self.push(object);
    } else {
      self.vec.insert(index, Some(Relationship::new(object)));
    }
  }

  /// Inserts the given object as a child Relationship at the given index,
  /// pushing further Relationships upward.
  ///
  /// Holes may be created if the index doesn't exist.
  pub fn insert_child(&mut self, index: uint, object: ObjectRef) {
    if index >= self.len() {
      self.expand_to(index);
      self.push_child(object);
    } else {
      self.vec.insert(index, Some(Relationship::new_child(object)));
    }
  }

  /// Removes the Relationship at the given index, pulling further Relationships
  /// downward to fill the gap, shrinking the list.
  ///
  /// If no Relationship exists at the index, `None` is returned. Otherwise,
  /// returns the removed Relationship.
  pub fn remove(&mut self, index: uint) -> Option<Relationship> {
    self.vec.remove(index).unwrap_or(None)
  }

  /// Deletes the Relationship at the given index, replacing it with a hole.
  /// Further Relationships are not affected.
  ///
  /// If no Relationship exists at the index, `None` is returned. Otherwise,
  /// returns the removed Relationship.
  pub fn delete(&mut self, index: uint) -> Option<Relationship> {
    if index >= self.len() {
      None
    } else {
      replace(self.vec.get_mut(index), None)
    }
  }

  /// Searches for a given key according to Paws' "nuclear" association-list
  /// semantics.
  ///
  /// Obeys the noughty rule: member 0 is not looked at.
  ///
  /// # Example
  ///
  /// Using JavaScript-like syntax (holes are represented as nothing) to
  /// represent members, ignoring other properties of the objects:
  ///
  ///     [, [, hello, world], [, foo, bar], [, hello, goodbye]]
  ///
  /// When looking up `hello`:
  ///
  /// * Iteration is done in reverse order; key and value are second and
  ///   third elements respectively, so result is `Some(goodbye)`
  pub fn lookup_pair(&self, key: &ObjectRef) -> Option<ObjectRef> {
    // Iterate through the members, looking for pair-shaped objects with
    // keys (1) that match the key we're looking for and get the value (2).
    for maybe_relationship in self.iter().rev() {
      match maybe_relationship {
        &Some(ref relationship) => {
          let object  = relationship.to().lock();
          let members = &object.meta().members;

          // Pair objects look approximately like [, key, value].
          match (members.get(1), members.get(2)) {
            (Some(rel_key), Some(rel_value)) => {
              if rel_key.to().eq_as_symbol(key) ||
                 rel_key.to() == key {
                return Some(rel_value.to().clone())
              }
            },
            _ => ()
          }
        },
        _ => ()
      }
    }
    None
  }

  /// Creates a pair out of the `key` and `value` and pushes it as a child
  /// Relationship to the pair only (not to the `value`).
  ///
  /// Enforces the noughty rule: if the members list is empty, a hole will be
  /// pushed first to avoid touching the 0th index.
  pub fn push_pair(&mut self, key: ObjectRef, value: ObjectRef) {
    let pair = Thing::pair(key, value);

    self.expand_to(1);
    self.push_child(pair);
  }

  /// Creates a pair out of the `key` and `value` and pushes it as a child
  /// Relationship to the pair, which itself has a child Relationship to the
  /// `value` (but not the `key`).
  ///
  /// Enforces the noughty rule: if the members list is empty, a hole will be
  /// pushed first to avoid touching the 0th index.
  pub fn push_pair_to_child(&mut self, key: ObjectRef, value: ObjectRef) {
    let pair = Thing::pair_to_child(key, value);

    self.expand_to(1);
    self.push_child(pair);
  }

  /// Creates holes to grow the list to the given size.
  pub fn expand_to(&mut self, size: uint) {
    self.vec.reserve(size);

    while self.len() < size {
      self.vec.push(None);
    }
  }
}

impl Collection for Members {
  fn len(&self) -> uint {
    self.vec.len()
  }
}
