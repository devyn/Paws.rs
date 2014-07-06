//! Procedures specific to `Execution`s.

#![allow(unused_variable)]
#![allow(missing_doc)]

use object::*;
use object::thing::Thing;
use object::alien::Alien;
use object::execution::Execution;

use machine::*;

use util::namespace::*;

/// Generates an `infrastructure execution` namespace object.
pub fn make(machine: &Machine) -> ObjectRef {
  let mut execution =
    box Thing::from_meta(Meta::with_receiver(namespace_receiver));

  {
    let mut add = NamespaceBuilder::new(machine, &mut *execution);

    add.call_pattern( "branch",                  branch, 1                    );

    add.call_pattern( "stage",                   stage, 2                     );
    add.call_pattern( "unstage",                 unstage, 0                   );
  }

  ObjectRef::new(execution).tag("(infra. execution)")
}

pub fn branch(machine: &Machine, caller: ObjectRef, args: &[ObjectRef])
              -> Reaction {
  match args {
    [ref executionish] => {
      let clone = match executionish.lock().try_cast::<Execution>() {

        Ok(execution) =>
          ObjectRef::new_clone_of(executionish, box execution.deref().clone()),

        Err(unknown) => match unknown.try_cast::<Alien>() {

          Ok(alien) =>
            ObjectRef::new_clone_of(executionish, box alien.deref().clone()),

          Err(_) => {
            warn!(concat!("tried to branch {}, which is neither",
                          " an execution nor an alien"),
                  executionish);

            return Yield
          }
        }
      };

      if &caller == executionish {
        debug!(concat!("branching caller: staging {} (caller) and {} (clone)",
                       " with each other, clone first"),
               caller, clone);

        // If we are branching the caller, react both the clone and the caller
        // with each other -- this ensures both proceed.
        machine.enqueue(caller.clone(), clone.clone());

        // Give priority to the clone.
        React(clone, caller)
      } else {
        debug!("branching {} (original) => {} (clone)", executionish, clone);

        React(caller, clone)
      }
    },
    _ => fail!("wrong number of arguments")
  }
}

pub fn stage(machine: &Machine, caller: ObjectRef, args: &[ObjectRef])
             -> Reaction {
  match args {
    [ref execution, ref response] => {
      // Put the caller on the queue so that...
      machine.enqueue(caller, execution.clone());

      // the execution gets priority by being the immediate result.
      React(execution.clone(), response.clone())
    },
    _ =>
      fail!("wrong number of arguments")
  }
}

pub fn unstage(machine: &Machine, caller: ObjectRef, args: &[ObjectRef])
               -> Reaction {
  Yield
}
