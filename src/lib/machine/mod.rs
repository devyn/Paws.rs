//! Paws machines and reactor implementation.

use script::Script;

use object::Object;
use object::ObjectRef;
use object::Relationship;

use object::empty::Empty;
use object::symbol::{Symbol, SymbolMap};
use object::execution::Execution;

use sync::{Arc, Mutex};

/// A machine represents the context of execution for Paws programs.
pub struct Machine {
  /// Dictates which pointers should be used to represent Symbol strings.
  pub  symbol_map: Arc<Mutex<SymbolMap>>,

  /// A symbol string for "locals" used internally to affix Executions' locals
  /// onto them, as well as for comparison. Purely an optimization to avoid
  /// locking the symbol map; not strictly necessary.
  priv locals_sym: Arc<~str>
}

impl Machine {
  /// Creates a new Machine.
  pub fn new() -> Machine {
    let mut symbol_map = SymbolMap::new();
    let     locals_sym = symbol_map.intern("locals");

    Machine {
      symbol_map: Arc::new(Mutex::new(symbol_map)),
      locals_sym: locals_sym
    }
  }

  /// Creates a `Symbol` object representing the given string within the context
  /// of this machine.
  ///
  /// This is the recommended way to create new Symbols.
  pub fn symbol(&self, string: &str) -> ObjectRef {
    ObjectRef::new_symbol(
      ~Symbol::new(self.symbol_map.lock().intern(string)))
  }

  /// Creates an Execution object from the given `Script` with a 'locals' member
  /// pointing at a new `Empty`.
  ///
  /// This is the recommended way to create new Executions.
  pub fn execution(&self, root: Script) -> ObjectRef {
    let mut execution = ~Execution::new(root);

    let locals_key = ObjectRef::new_symbol(
                       ~Symbol::new(self.locals_sym.clone()));

    let locals_ref = ObjectRef::new(~Empty::new());

    let locals_pair =
      ObjectRef::new(
        ~Empty::new_pair_to_child(locals_key, locals_ref));

    execution.meta_mut().members.push(None);
    execution.meta_mut().members.push(Some(
      Relationship::new_child(locals_pair)));

    ObjectRef::new(execution)
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
