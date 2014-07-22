//! Machines hold necessary global state for an entire Paws world.
//!
//! They may have `Reactor`s operating within their context, which are the
//! evaluation cores of Paws.

use script::Script;

use object::Object;
use object::ObjectRef;

use object::symbol::{Symbol, SymbolMap};
use object::execution::Execution;
use object::locals::Locals;

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
    let     locals_sym = ObjectRef::new_symbol(
                           box Symbol::new(symbol_map.intern("locals")));

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
    ObjectRef::new_symbol(
      box Symbol::new(self.symbol_map.lock().intern(string)))
  }

  /// Creates an Execution object from the given `Script` with a 'locals' member
  /// pointing at a new `Locals` named "locals".
  ///
  /// This is the recommended way to create new Executions.
  pub fn execution(&self, root: Script) -> ObjectRef {
    let mut execution = box Execution::new(root);

    let locals_key = ObjectRef::new_symbol(box Symbol::new(
                       self.locals_sym.symbol_ref().unwrap().clone()));

    let locals_ref = ObjectRef::new(box Locals::new(self.locals_sym.clone()));

    execution.meta_mut().members.push_pair_to_child(locals_key, locals_ref);

    ObjectRef::new(execution)
  }

  /// Exposes the system interface (`infrastructure` and `implementation`) as
  /// members of the locals of the given Execution.
  pub fn expose_system_to(&self, execution: &mut Execution) {
    let System {
          infrastructure: infrastructure,
          implementation: implementation
        } = self.system();

    let     locals_ref = execution.meta_mut().members
                           .lookup_pair(&self.locals_sym).unwrap();
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
