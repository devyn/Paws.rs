//! Paws machines and reactor implementation.

use script::Script;

use object::Object;
use object::ObjectRef;
use object::ObjectRefGuard;
use object::{Reaction, React, Yield};
use object::{NativeReceiver, ObjectReceiver};
use object::Params;

use object::thing::Thing;
use object::symbol::{Symbol, SymbolMap};
use object::execution::Execution;
use object::alien::Alien;
use object::locals::Locals;

use system::implementation;
use system::infrastructure;

use util::queue::Queue;

use std::any::AnyRefExt;
use sync::{Arc, Mutex};

#[cfg(test)]
mod tests;

/// A machine represents the context of execution for Paws programs.
#[deriving(Clone)]
pub struct Machine {
  /// Dictates which pointers should be used to represent Symbol strings.
  pub symbol_map: Arc<Mutex<SymbolMap>>,

  /// A Symbol for "locals" used internally to affix Executions' locals onto
  /// them, as well as for comparison. Purely an optimization to avoid locking
  /// the symbol map; not strictly necessary.
      locals_sym: ObjectRef,

  /// The receive-end of the main execution realization queue. Reactors pull
  /// from this.
      queue:      Arc<Queue<Realization>>,

  /// The system interface. See `paws::system`. Lazily generated, because many
  /// tests don't need it.
      system:     Arc<Mutex<Option<System>>>,
}

impl Machine {
  /// Creates a new Machine.
  pub fn new() -> Machine {
    let mut symbol_map = SymbolMap::new();
    let     locals_sym = ObjectRef::new_symbol(
                           box Symbol::new(symbol_map.intern("locals")));

    Machine {
      symbol_map: Arc::new(Mutex::new(symbol_map)),
      locals_sym: locals_sym,

      queue:      Arc::new(Queue::new()),

      system:     Arc::new(Mutex::new(None))
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

  /// Adds a realization (execution and response) to the machine's global queue,
  /// which the Machine's reactors should soon pick up as long as there is
  /// nothing blocking that execution.
  ///
  /// **Note:** if you're calling this from a receiver or Alien, you probably
  /// want to `React` instead; it's more efficient, but with a few caveats.
  pub fn enqueue(&self, execution: ObjectRef, response: ObjectRef) {
    self.queue.push(Realization(execution, response));
  }

  /// Gets a realization from the machine's queue, blocking until either one is
  /// available (in which case `Some(Realization)` is returned), or the
  /// `Machine` has been `stop()`ped (in which case `None` is returned).
  pub fn dequeue(&self) -> Option<Realization> {
    self.queue.shift()
  }

  /// Marks the machine's global queue for termination. This action is
  /// irreversable.
  ///
  /// Reactors may not stop immediately, but they should stop as soon as they
  /// check the global queue.
  pub fn stop(&self) {
    debug!("*** machine stopping ***");
    self.queue.end();
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
  pub fn combine<'a>
                 (&self,
                 caller: ObjectRefGuard<'a>,
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
        match caller.deref().meta().members.lookup_pair(&self.locals_sym) {
          Some(locals) => (locals, message),
          None         => return Yield
        }
      }
    };

    // We no longer need to look at any of the caller's properties.
    let caller = caller.unlock().clone();

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
        // If the receiver is a NativeReceiver, then call the function it
        // contains.
        NativeReceiver(function) => {
          drop(current_target); // Release the lock ASAP.

          return function(self, Params {
            caller:  caller,
            subject: subject,
            message: message
          })
        },

        // Otherwise, we need to check if this receiver is queueable (Execution
        // or Alien) or not.
        ObjectReceiver(receiver) => {
          drop(current_target); // Release the lock ASAP.

          let queueable = {
            let receiver = receiver.lock();
            receiver.deref().is::<Execution>() ||
            receiver.deref().is::<Alien>()
          };

          if queueable {
            // If it is, we construct a params object
            // `[, caller, subject, message]` and `React` the receiver with the
            // params object as the response.
            let mut params = box Thing::new();

            params.meta_mut().members.set(1, caller);
            params.meta_mut().members.set(2, subject);
            params.meta_mut().members.set(3, message);

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

  /// Iterates over the machine's global queue, performing realizations.
  ///
  /// Multiple reactors may be run as separate native tasks (**important!** not
  /// green-threading compatible at the moment), or a single reactor setup may be
  /// run standalone in any task.
  pub fn run_reactor(&self) {

    debug!("start reactor");

    'queue:
    for Realization(mut execution_ref, mut response_ref) in self.queue.iter() {

      'immediate:
      loop {
        // Detect whether `execution_ref` is an Execution, an Alien, or
        // something else, and handle those cases separately, capturing the
        // Reaction.
        let reaction = match execution_ref.lock().try_cast::<Execution>() {
          Ok(mut execution) => {
            // For an Execution, we just want to advance() it and have the
            // Machine process the combination if there was one.

            debug!("realize execution {} \t<-- {}",
              execution_ref, response_ref);

            match execution.advance(execution_ref.clone(), response_ref) {
              Some(combination) =>
                // Calls the receiver and all that jazz, resulting in a
                // Reaction.
                self.combine(execution.into_untyped(), combination),

              None => {
                // This execution is already complete, so we can't do anything;
                // we have to go back to the queue.

                debug!("yield reactor: execution complete");
                continue 'queue
              }
            }
          },

          Err(execution_ish) =>
            match execution_ish.try_cast::<Alien>() {
              Ok(alien) => {
                // Aliens are a bit different. They handle unlocking themselves
                // at a point which they see fit, so we give them the lock.

                debug!("realize alien     {} \t<-- {}",
                  execution_ref, response_ref);

                Alien::realize(alien, self, response_ref)
              },

              Err(_) => {
                // Finally, if it was neither an Execution nor an Alien, it
                // really doesn't belong in this queue and we'll just pretend it
                // wasn't there.
                
                warn!("yield reactor: tried to realize non-queueable {}!",
                  execution_ref);
                continue 'queue
              }
            }
        };

        // Handle the Reaction.
        match reaction {
          React(next_execution_ref, next_response_ref) => {
            // We got an execution and response right away, so let's do that
            // immediately.
            execution_ref = next_execution_ref;
            response_ref  = next_response_ref;
            continue 'immediate
          },

          Yield => {
            // The receiver or Alien wants us to go back to the queue,
            // potentially because it doesn't have anything ready for us right
            // now or because it intentionally doesn't want to continue.

            debug!("yield reactor: explicit");
            continue 'queue
          }
        }
      }
    }
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

/// Describes a Combination of a `message` against a `subject`.
///
/// If the `subject` is `None`, the Combination shall be against the calling
/// Execution's locals.
#[deriving(Clone, PartialEq, Eq, Show)]
pub struct Combination {
  /// The left hand side, what the `message` is combined *against*.
  pub subject: Option<ObjectRef>,

  /// The right hand side, what the `subject` is combined *with*.
  pub message: ObjectRef
}

/// Describes a Realization of an `execution` (0) with a `response` (1).
#[deriving(Clone, PartialEq, Eq)]
pub struct Realization(pub ObjectRef, pub ObjectRef);

/// The system interface.
#[deriving(Clone)]
struct System {
  infrastructure: ObjectRef,
  implementation: ObjectRef
}
