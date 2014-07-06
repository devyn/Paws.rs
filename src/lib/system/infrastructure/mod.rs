//! Nucleus' standardized aliens for manipulating objects and the Machine.
//!
//! Because everything under the `infrastructure` namespace is standardized,
//! documentation will not be provided here for aliens, unless they have some
//! unusual Paws.rs-specific construction pattern.

#![allow(unused_variable)]
#![allow(missing_doc)]

use object::*;
use object::thing::Thing;

use machine::*;

use util::namespace::*;

pub mod execution;

/// Generates an `infrastructure` namespace object.
pub fn make(machine: &Machine) -> ObjectRef {
  let mut infrastructure =
    box Thing::from_meta(Meta::with_receiver(namespace_receiver));

  {
    let mut add = NamespaceBuilder::new(machine, &mut *infrastructure);

    add.namespace(    "execution",               execution::make              );
    add.call_pattern( "affix",                   affix, 2                     );
    add.call_pattern( "unaffix",                 unaffix, 1                   );
  }

  ObjectRef::new(infrastructure)
}

pub fn affix(machine: &Machine, caller: ObjectRef, args: &[ObjectRef])
             -> Reaction {
  match args {
    [ref onto, ref what] => {
      let mut onto = onto.lock();
      onto.meta_mut().members.push(what.clone());
      Yield
    },
    _ => fail!("wrong number of arguments")
  }
}

pub fn unaffix(machine: &Machine, caller: ObjectRef, args: &[ObjectRef])
               -> Reaction {
  match args {
    [ref from] => {
      let mut from = from.lock();

      match from.meta_mut().members.pop() {
        Some(relationship) => React(caller, relationship.unwrap()),
        None               => Yield
      }
    },
    _ => fail!("wrong number of arguments")
  }
}
