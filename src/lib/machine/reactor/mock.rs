use super::Reactor;

use machine::Machine;

use object::{ObjectRef, Cache};

/// A fake reactor that, instead of actually reacting anything, instead simply
/// accumulates state from the calls made to it.
pub struct MockReactor {
  /// Indicates whether the reactor is alive. This is `true` when created, but
  /// `false` as soon as `stop()` is called.
  ///
  /// No changes will be made to the reactor if it is not alive.
  pub alive:          bool,

  /// A log of all `stage()` calls made while the reactor was alive.
  pub stagings:       Vec<(ObjectRef, ObjectRef)>,

  /// A log of all `on_stall()` calls made while the reactor was alive.
  pub stall_handlers: Vec<proc (&mut Reactor)>,

  /// The machine associated with the reactor.
  pub machine:        Machine,

  /// The reactor's cache.
  pub cache:          Cache
}

impl MockReactor {
  /// Creates a new `MockReactor` for the given `Machine`.
  pub fn new(machine: Machine) -> MockReactor {
    MockReactor {
      alive:          true,
      stagings:       Vec::new(),
      stall_handlers: Vec::new(),
      machine:        machine,
      cache:          Cache::new_serial()
    }
  }
}

impl Reactor for MockReactor {
  fn stage(&mut self, execution: ObjectRef, response: ObjectRef) {
    if self.alive {
      self.stagings.push((execution, response));
    }
  }

  fn on_stall(&mut self, handler: proc (&mut Reactor)) {
    if self.alive {
      self.stall_handlers.push(handler);
    }
  }

  fn stop(&mut self) {
    self.alive = false;
  }

  fn machine(&self) -> &Machine {
    &self.machine
  }

  fn cache(&mut self) -> &mut Cache {
    &mut self.cache
  }
}
