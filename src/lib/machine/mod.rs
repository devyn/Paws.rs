//! Paws machines and reactor implementation.

use object::ObjectRef;
use object::symbol::{Symbol, SymbolMap};

use sync::{Arc, Mutex};

/// A machine represents the context of execution for Paws programs.
pub struct Machine {
  /// Dictates which pointers should be used to represent Symbol strings.
  pub symbol_map: Arc<Mutex<SymbolMap>>
}

impl Machine {
  /// Creates a new Machine.
  pub fn new() -> Machine {
    Machine {
      symbol_map: Arc::new(Mutex::new(SymbolMap::new()))
    }
  }

  /// Creates a Symbol object representing the given string within the context
  /// of this machine.
  pub fn symbol(&self, string: &str) -> ObjectRef {
    ObjectRef::new_symbol(
      ~Symbol::new(self.symbol_map.lock().intern(string)))
  }

  /// **TODO**. Adds an entry to the Machine's queue, making it available for a
  /// reactor to pull and execute.
  #[allow(unused_variable)]
  pub fn stage(&self, execution: ObjectRef, response: ObjectRef,
               mask: Option<MaskRequest>) {
    unimplemented!()
  }
}

/// Describes a Combination of a `message` against a `subject`.
///
/// If the `subject` is `None`, the Combination shall be against the calling
/// Execution's locals.
pub struct Combination {
  pub subject: Option<ObjectRef>,
  pub message: ObjectRef
}

/// **TODO**. A request for a mask.
///
/// No idea what this is going to look like yet.
#[deriving(Clone, Eq, TotalEq)]
pub struct MaskRequest;

/// **WIP**. An operation for a Reactor to react.
#[deriving(Clone, Eq, TotalEq)]
pub enum Operation {
  Stage(StageParams)
}

/// Parameters for a Stage operation.
#[deriving(Clone, Eq, TotalEq)]
pub struct StageParams {
  pub execution: ObjectRef,
  pub response:  ObjectRef,
  pub mask:      Option<MaskRequest>
}
