//! Procedures specific to `Execution`s.

#![allow(unused_variable)]
#![allow(missing_doc)]

use object::*;
use object::thing::Thing;

use machine::*;

use util::clone;
use util::namespace::*;

/// Generates an `infrastructure execution` namespace object.
pub fn make(machine: &Machine) -> ObjectRef {
  let mut execution = box Thing::new();

  {
    let mut add = NamespaceBuilder::new(machine, &mut *execution);

    add.call_pattern( "branch",                  branch, 1                    );

    add.call_pattern( "stage",                   stage, 2                     );
    add.call_pattern( "unstage",                 unstage, 0                   );
  }

  ObjectRef::new_with_tag(execution, "(infra. execution)")
}

pub fn branch(reactor: &mut Reactor, caller: ObjectRef, args: &[ObjectRef]) {
  match args {
    [ref executionish] =>
      match clone::queueable(executionish, reactor.machine()) {

        Some(clone) => reactor.stage(caller, clone),

        None =>
          warn!(concat!("tried to branch {}, which is neither",
                        " an execution nor an alien"),
                executionish)
      },
    _ => fail!("wrong number of arguments")
  }
}

pub fn stage(reactor: &mut Reactor, caller: ObjectRef, args: &[ObjectRef]) {
  match args {
    [ref execution, ref response] => {
      reactor.stage(execution.clone(), response.clone());
      reactor.stage(caller, execution.clone());
    },
    _ =>
      fail!("wrong number of arguments")
  }
}

pub fn unstage(reactor: &mut Reactor, caller: ObjectRef, args: &[ObjectRef]) {
  // Do nothing! :D
}
