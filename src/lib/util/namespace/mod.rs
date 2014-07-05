//! Utilities pertaining to 'namespace' objects, which are currently just Things
//! with a custom receiver.

use object::*;
use object::thing::Thing;
use object::alien::{Alien, CallPatternRoutine};

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
      ObjectRef::new(box factory(self.machine))
    );
  }

  /// Adds a new call pattern Alien with the given name.
  pub fn call_pattern(&mut self,
                      name:    &str,
                      routine: CallPatternRoutine,
                      n_args:  uint) {

    self.thing.meta_mut().members.push_pair_to_child(
      self.machine.symbol(name),
      ObjectRef::new(box Alien::new_call_pattern(routine, n_args))
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

/// Similar to `object::lookup_receiver()`, but clones any `Alien` members
/// before returning them.
#[allow(unused_variable)]
pub fn namespace_receiver(machine: &Machine, params: Params) -> Reaction {
  let lookup_result = {
    let subject = params.subject.lock();

    subject.deref().meta().members.lookup_pair(&params.message)
  };

  debug!("{} <namespace_receiver> {} => {}",
    params.subject, params.message, lookup_result);

  match lookup_result {
    Some(value) =>
      match value.lock().try_cast::<Alien>() {
        Ok(alien) =>
          React(params.caller.clone(),
                ObjectRef::new(box alien.deref().clone())),
        Err(object) =>
          React(params.caller.clone(), object.unlock().clone())
      },
    None =>
      Yield
  }
}
