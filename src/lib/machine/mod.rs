//! Machines hold necessary global state for an entire Paws world.
//!
//! They may have `Reactor`s operating within their context, which are the
//! evaluation cores of Paws.

use object::ObjectRef;

use nuketype::symbol::{Symbol, SymbolMap};

use system::implementation;
use system::infrastructure;

use std::sync::{Arc, Mutex};

pub use self::reactor::Reactor;
pub use self::reactor::Combination;

pub mod reactor;

#[cfg(test)]
mod tests;

/// A machine represents the context of execution for Paws programs.
#[deriving(Clone)]
pub struct Machine {
  /// Dictates which pointers should be used to represent Symbol strings.
  pub symbol_map:     Arc<Mutex<SymbolMap>>,

  /// A Symbol for "locals" used internally to affix Executions' locals onto
  /// them, as well as for comparison. Purely an optimization to avoid locking
  /// the symbol map; not strictly necessary.
  pub locals_sym:     ObjectRef,

  /// The system interface. See `paws::system`. Lazily generated, because many
  /// tests don't need it.
      system:         Arc<Mutex<Option<System>>>,
}

impl Machine {
  /// Creates a new Machine.
  pub fn new() -> Machine {
    let mut symbol_map = SymbolMap::new();
    let     locals_sym = Symbol::create(symbol_map.intern("locals"));

    Machine {
      symbol_map:     Arc::new(Mutex::new(symbol_map)),
      locals_sym:     locals_sym,
      system:         Arc::new(Mutex::new(None))
    }
  }

  /// Creates a `Symbol` object representing the given string within the context
  /// of this machine.
  ///
  /// This is the recommended way to create new Symbols.
  pub fn symbol(&self, string: &str) -> ObjectRef {
    Symbol::create(self.symbol_map.lock().intern(string))
  }

  /// Exposes the system interface (`infrastructure` and `implementation`) as
  /// members of the locals of the given Execution.
  pub fn expose_system_to(&self, execution: &ObjectRef) {
    let System {
          infrastructure: infrastructure,
          implementation: implementation
        } = self.system();

    let     locals_ref = execution.lock().meta().members
                           .lookup_pair(&self.locals_sym)
                           .expect("Execution is missing locals!");
    let mut locals_obj = locals_ref.lock();
    let     locals     = &mut locals_obj.meta_mut().members;

    locals.push_pair(self.symbol("infrastructure"), infrastructure);
    locals.push_pair(self.symbol("implementation"), implementation);
  }

  /// Lazy-get the system interface.
  fn system(&self) -> System {
    let mut lazy_system = self.system.lock();

    match lazy_system.clone() {
      Some(system) =>
        system,

      None => {
        let system = System {
          infrastructure: infrastructure::make(self),
          implementation: implementation::make(self)
        };

        *lazy_system = Some(system.clone());

        system
      }
    }
  }
}

/// The system interface.
#[deriving(Clone)]
struct System {
  infrastructure: ObjectRef,
  implementation: ObjectRef
}
