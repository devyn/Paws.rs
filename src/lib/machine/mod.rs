//! Paws machines and reactor implementation.

use script::Script;

use object::Object;
use object::ObjectRef;
use object::Relationship;
use object::{Reaction, React, Yield};
use object::Params;

use object::empty::Empty;
use object::symbol::{Symbol, SymbolMap};
use object::execution::Execution;
use object::alien::Alien;

use std::any::AnyRefExt;
use sync::{Arc, Mutex};

#[cfg(test)]
mod tests;

/// A machine represents the context of execution for Paws programs.
pub struct Machine {
  /// Dictates which pointers should be used to represent Symbol strings.
  pub  symbol_map: Arc<Mutex<SymbolMap>>,

  /// A Symbol for "locals" used internally to affix Executions' locals
  /// onto them, as well as for comparison. Purely an optimization to avoid
  /// locking the symbol map; not strictly necessary.
  priv locals_sym: ObjectRef
}

impl Machine {
  /// Creates a new Machine.
  pub fn new() -> Machine {
    let mut symbol_map = SymbolMap::new();
    let     locals_sym = ObjectRef::new_symbol(
                           ~Symbol::new(symbol_map.intern("locals")));

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

    let locals_key = ObjectRef::new_symbol(~Symbol::new(
                         self.locals_sym.symbol_ref().unwrap().clone()));

    let locals_ref = ObjectRef::new(~Empty::new());

    let locals_pair =
      ObjectRef::new(
        ~Empty::new_pair_to_child(locals_key, locals_ref));

    execution.meta_mut().members.push(None);
    execution.meta_mut().members.push(Some(
      Relationship::new_child(locals_pair)));

    ObjectRef::new(execution)
  }

  /// Implements the combination algorithm, finding the appropriate receiver and
  /// then invoking it.
  ///
  /// If the `combination`'s `subject` is `None`, it will be interpreted to be
  /// the `caller`'s "locals".
  ///
  /// From the spec:
  ///
  /// > **Finding the receiver for a given Object**
  /// >
  /// > A 'receiver' is an Execution associated with a given object, one
  /// responsible for handling combinations when that object is the `subject` of
  /// the combination.
  /// >
  /// > 1. If the `subject` has no `receiver` property set, then an Execution
  /// >    implementing the "default receiver" algorithm for that type of object
  /// >    is the result of this algorithm. (Each type described above includes
  /// >    a description of that type's default receiver's algorithm.)
  /// >
  /// > 2. If the `subject` has a `receiver` property, and the value of that
  /// >    property is queueable (i.e. an Execution), then that Execution is the
  /// >    result of this algorithm.
  /// >
  /// > 3. If the `subject`'s `receiver` is not queueable (that is, not an
  /// >    Execution), then recursively apply this algorithm starting at 1, with
  /// >    that `receiver` as the `subject` *for the purposes of this
  /// >    algorithm*. (*Not* for the consumer of this algorithm, who will have
  /// >    their own reference to the original `subject`.)
  /// >
  /// > **Rationale:** The recursive nature of this process allows object-system
  /// > designers to wrap their receiver(s) in metadata, or otherwise abstract
  /// > them away.
  pub fn combine(&self,
                 caller: ObjectRef,
                 combination: Combination)
                 -> Reaction {

    // Get the actual subject and message, interpreting a None subject in the
    // combination provided as "locals".
    let (subject, message) = match combination {
      Combination { subject: Some(subject),
                    message: message } =>
        (subject, message),

      Combination { subject: None,
                    message: message } => {

        // Find the caller's locals and make that the subject.
        //
        // If we can't find the locals, immediately give up and return Yield --
        // we can't continue, since the Execution is obviously totally fucked
        // up. Yes, there should be some error reporting, but that's not how
        // this works at the moment.
        match caller.lock().meta().lookup_member(&self.locals_sym) {
          Some(locals) => (locals, message),
          None         => return Yield
        }
      }
    };

    // Perform the receiver-finding algorithm, using `use_receiver_of` to
    // iterate through until we find the receiver we want to use.
    let mut use_receiver_of = subject.clone();
    loop {
      // We have to clone this again because rustc apparently isn't smart enough
      // to realize that `drop(current_target)` means that use_receiver_of is no
      // longer borrowed >_>
      let current_target_ref = use_receiver_of.clone();
      let current_target     = current_target_ref.lock();

      match current_target.deref().meta().receiver.clone() {
        // If the receiver is None, then we want to use this object's default
        // receiver.
        None => {
          let receiver = current_target.deref().default_receiver();

          drop(current_target); // Release the lock ASAP.

          return receiver(self, Params {
            caller:  caller,
            subject: subject,
            message: message
          })
        },

        // Otherwise, we need to check if this receiver is queueable (Execution
        // or Alien) or not.
        Some(receiver) => {
          drop(current_target); // Release the lock ASAP.

          let queueable = {
            let receiver = receiver.lock();
            receiver.deref().as_any().is::<Execution>() ||
            receiver.deref().as_any().is::<Alien>()
          };

          if queueable {
            // If it is, we construct a params object
            // `[, caller, subject, message]` and `React` the receiver with the
            // params object as the response.
            let mut params = ~Empty::new();

            params.meta_mut().members.push(None);
            params.meta_mut().members.push(Some(Relationship::new(caller)));
            params.meta_mut().members.push(Some(Relationship::new(subject)));
            params.meta_mut().members.push(Some(Relationship::new(message)));

            return React(receiver.clone(), ObjectRef::new(params))
          } else {
            // If it isn't, we need to loop through this whole thing again, with
            // this receiver as `use_receiver_of`.
            use_receiver_of = receiver;
          }
        }
      }
    }
  }
}

/// Describes a Combination of a `message` against a `subject`.
///
/// If the `subject` is `None`, the Combination shall be against the calling
/// Execution's locals.
pub struct Combination {
  /// The left hand side, what the `message` is combined *against*.
  pub subject: Option<ObjectRef>,

  /// The right hand side, what the `subject` is combined *with*.
  pub message: ObjectRef
}

/// **TODO**. A request for a mask.
///
/// No idea what this is going to look like yet.
#[deriving(Clone, Eq, TotalEq)]
pub struct MaskRequest;

/// Parameters for a realization operation.
#[deriving(Clone, Eq, TotalEq)]
pub struct RealizeParams {
  /// The execution to realize.
  pub execution: ObjectRef,

  /// The response to realize the execution with, filling the hole that was left
  /// in it.
  pub response:  ObjectRef,

  /// A request for responsibility over some object that must be fulfilled, if
  /// present, before realizing the execution.
  pub mask:      Option<MaskRequest>
}
