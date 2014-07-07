//! Utilities pertaining to 'namespace' objects, which are currently just Things
//! with a custom receiver.

use object::*;
use object::thing::Thing;
use object::alien::{Alien, CallPatternRoutine, OneshotRoutine};

use machine::*;

/// Generates namespaces.
pub struct NamespaceBuilder<'a> {
  machine: &'a Machine,
  thing:   &'a mut Thing
}

impl<'a> NamespaceBuilder<'a> {
  /// Creates a new NamespaceBuilder wrapping the given Thing for the Machine.
  pub fn new(machine: &'a Machine,
             thing:   &'a mut Thing)
             -> NamespaceBuilder<'a> {

    NamespaceBuilder {
      machine: machine,
      thing:   thing
    }
  }

  /// Adds a new Alien from a factory function with the given name.
  pub fn factory(&mut     self,
                 name:    &str,
                 factory: fn (&Machine) -> Alien) {

    self.thing.meta_mut().members.push_pair_to_child(
      self.machine.symbol(name),
      ObjectRef::new(box factory(self.machine)).tag(name)
    );
  }

  /// Adds a new call pattern Alien with the given name.
  pub fn call_pattern(&mut self,
                      name:    &str,
                      routine: CallPatternRoutine,
                      n_args:  uint) {

    self.thing.meta_mut().members.push_pair_to_child(
      self.machine.symbol(name),
      ObjectRef::new(box Alien::new_call_pattern(routine, n_args)).tag(name)
    );
  }

  /// Adds a new oneshot Alien with the given name.
  pub fn oneshot(&mut self,
                 name:    &str,
                 routine: OneshotRoutine) {

    self.thing.meta_mut().members.push_pair_to_child(
      self.machine.symbol(name),
      ObjectRef::new(box Alien::new_oneshot(routine)).tag(name)
    );
  }

  /// Adds a new namespace with the given name, from a `make()` function.
  pub fn namespace(&mut self,
                   name: &str,
                   make: fn (&Machine) -> ObjectRef) {

    self.thing.meta_mut().members.push_pair_to_child(
      self.machine.symbol(name),
      make(self.machine)
    );
  }
}
