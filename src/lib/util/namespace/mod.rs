//! Utilities pertaining to 'namespace' objects, which are currently just Things
//! with a custom receiver.

use object::{ObjectRef, Meta};

use nuketype::alien::Alien;
use nuketype::alien::CallPatternRoutine;
use nuketype::alien::OneshotRoutine;

use machine::Machine;

/// Generates namespaces.
pub struct NamespaceBuilder<'a> {
  machine: &'a Machine,
  meta:    &'a mut Meta
}

impl<'a> NamespaceBuilder<'a> {
  /// Creates a new NamespaceBuilder wrapping the given Meta for the Machine.
  pub fn new(machine: &'a Machine,
             meta:    &'a mut Meta)
             -> NamespaceBuilder<'a> {

    NamespaceBuilder {
      machine: machine,
      meta:    meta
    }
  }

  /// Adds a new object from a factory function with the given name.
  pub fn factory(&mut     self,
                 name:    &str,
                 factory: fn (&Machine) -> ObjectRef) {

    self.meta.members.push_pair_to_child(
      self.machine.symbol(name),
      factory(self.machine)
    );
  }

  /// Adds a new call pattern Alien with the given name.
  pub fn call_pattern(&mut self,
                      name:    &str,
                      routine: CallPatternRoutine,
                      n_args:  uint) {

    self.meta.members.push_pair_to_child(
      self.machine.symbol(name),
      Alien::call_pattern(name, routine, n_args)
    );
  }

  /// Adds a new oneshot Alien with the given name.
  pub fn oneshot(&mut self,
                 name:    &str,
                 routine: OneshotRoutine) {

    self.meta.members.push_pair_to_child(
      self.machine.symbol(name),
      Alien::oneshot(name, routine)
    );
  }
}
