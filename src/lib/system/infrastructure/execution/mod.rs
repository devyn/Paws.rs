//! Procedures specific to `Execution`s.

#![allow(unused_variable)]
#![allow(missing_doc)]

use object::*;
use object::thing::Thing;

use machine::*;

use util::namespace::*;

/// Generates an `infrastructure execution` namespace object.
pub fn make(machine: &Machine) -> ObjectRef {
  let mut execution =
    box Thing::from_meta(Meta::with_receiver(namespace_receiver));

  {
    let mut add = NamespaceBuilder::new(machine, &mut *execution);

    add.call_pattern( "stage",                   stage, 2                     );
  }

  ObjectRef::new(execution).tag("(infra. execution)")
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
