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
use object::locals::Locals;

use system::implementation;
use system::infrastructure;

use util::work_queue::{WorkQueue, Work, Stalled, Ended};
use util::clone;

use std::mem::replace;

use std::sync::{Arc, Mutex};

pub use util::work_queue::WorkGuard;

pub use machine::reactor::Reactor;

mod reactor;

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

  /// The receive-end of the main execution realization queue. Reactors pull
  /// from this.
      queue:          Arc<WorkQueue<Realization>>,

  /// The system interface. See `paws::system`. Lazily generated, because many
  /// tests don't need it.
      system:         Arc<Mutex<Option<System>>>,

  /// Stall handler procedures that are called one time if a stall occurs.
      stall_handlers: Arc<Mutex<Vec<proc(&Machine): Send>>>,
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

      queue:          Arc::new(WorkQueue::new()),

      system:         Arc::new(Mutex::new(None)),

      stall_handlers: Arc::new(Mutex::new(Vec::new()))
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
  /// available (in which case `Some(WorkGuard<Realization>)` is returned), or
  /// the `Machine` has been `stop()`ped (in which case `None` is returned).
  ///
  /// It is important that the `WorkGuard` be kept around while work is being
  /// done that was requested from the queue. This allows the Machine to detect
  /// stalls and handle them appropriately.
  pub fn dequeue<'a>(&'a self) -> Option<WorkGuard<'a, Realization>> {
    loop {
      match self.queue.shift() {
        Work(work) => return Some(work),
        Ended      => return None,
        Stalled    => {
          // Call our stall handlers...
          let handlers = replace(&mut *self.stall_handlers.lock(), Vec::new());

          for handler in handlers.move_iter() {
            handler(self);
          }

          // ...and retry.
          continue
        }
      }
    }
  }

  /// Creates an iterator over the work in the queue, ending when the Machine
  /// `stop()`s.
  pub fn iter_queue<'a>(&'a self) -> WorkItems<'a> {
    WorkItems { machine: self }
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

          match clone::queueable(&receiver) {
            Some(clone) => {
              // If it is, we construct a params object `[, caller, subject,
              // message]` and `React` a clone of the receiver with the params
              // object as the response.
              //
              // TODO: Find a way to not have to clone it all the time.
              let mut params = box Thing::new();

              params.meta_mut().members.set(1, caller);
              params.meta_mut().members.set(2, subject);
              params.meta_mut().members.set(3, message);

              return React(clone, ObjectRef::new(params))
            },

            None => {
              // If it isn't, we need to loop through this whole thing again, with
              // this receiver as `use_receiver_of`.
              use_receiver_of = receiver;
            }
          }
        }
      }
    }
  }

  /// Adds a handler to be called if the Machine stalls.
  ///
  /// The handler will only be called for a single stall, so if it is intended
  /// to run more than once, it should re-add itself as a new unique procedure.
  ///
  /// It also needs to be `Send`able because it is possible (and often quite
  /// likely) that the task that the procedure is run on won't be the same one
  /// it was created from.
  pub fn on_stall(&self, handler: proc(&Machine): Send) {
    self.stall_handlers.lock().push(handler);
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

/// An iterator over work in a machine's queue.
pub struct WorkItems<'a> {
  machine: &'a Machine
}

impl<'a> Iterator<WorkGuard<'a, Realization>> for WorkItems<'a> {
  fn next(&mut self) -> Option<WorkGuard<'a, Realization>> {
    self.machine.dequeue()
  }
}
