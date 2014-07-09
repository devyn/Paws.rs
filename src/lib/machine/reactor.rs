use machine::Machine;
use machine::Realization;

use object::{React, Yield};
use object::execution::Execution;
use object::alien::Alien;

use std::any::Any;
use std::sync::Future;
use std::task::{TaskBuilder};

use native::NativeTaskBuilder;

/// A single Paws reactor.
///
/// Responsible a single Machine's Unit. In the future, Machines will be split
/// so that they can have multiple Units and also talk to other Machines,
/// possibly on a network.
pub struct Reactor {
  machine: Machine
}

impl Reactor {
  /// Creates a new Reactor to operate on the given Machine.
  pub fn new(machine: Machine) -> Reactor {
    Reactor { machine: machine }
  }

  /// Runs the reactor loop in the current task.
  pub fn run(&self) {

    debug!("start reactor");

    'queue:
    for Realization(mut execution_ref, mut response_ref)
        in self.machine.iter_queue() {

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
                self.machine.combine(execution.into_untyped(), combination),

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

                Alien::realize(alien, &self.machine, response_ref)
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

    debug!("stop reactor");
  }

  /// Spawns a new native (non-green) task running this reactor.
  ///
  /// Returns a `Future` that can be used to block until the reactor stops or
  /// fails, if desired.
  pub fn spawn(self) -> Future<Result<(), Box<Any + Send>>> {
    TaskBuilder::new().native().try_future(proc() {
      self.run();
    })
  }
}
