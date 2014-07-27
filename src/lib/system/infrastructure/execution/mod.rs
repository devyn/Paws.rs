//! Procedures specific to `Execution`s.

#![allow(unused_variable)]
#![allow(missing_doc)]

use object::{ObjectRef, Meta};

use nuketype::Thing;

use machine::{Machine, Reactor};

use util::namespace::NamespaceBuilder;
use util::clone;

/// Generates an `infrastructure execution` namespace object.
pub fn make(machine: &Machine) -> ObjectRef {
  let mut execution = Meta::new();

  {
    let mut add = NamespaceBuilder::new(machine, &mut execution);

    add.call_pattern( "branch",                  branch, 1                    );

    add.call_pattern( "stage",                   stage, 2                     );
    add.oneshot(      "unstage",                 unstage                      );
  }

  Thing::tagged(execution, "(infra. execution)")
}

pub fn branch(reactor: &mut Reactor, caller: ObjectRef, args: &[ObjectRef]) {
  match args {
    [ref executionish] => {
      let locals_sym = reactor.machine().locals_sym.clone();

      match clone::stageable(executionish, &locals_sym) {

        Some(clone) => reactor.stage(caller, clone),

        None =>
          warn!(concat!("tried to branch {}, which is neither",
                        " an execution nor an alien"),
                executionish)
      }
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

pub fn unstage(reactor: &mut Reactor, response: ObjectRef) {
  // Do nothing! :D
}
