//! Nucleus' standardized aliens for manipulating objects and the Machine.

#![allow(unused_variable)]

use object::*;
use object::thing::Thing;

use machine::*;

use util::namespace::*;

/// Generates an `infrastructure` namespace object.
pub fn make(machine: &Machine) -> ObjectRef {
  let mut infrastructure =
    box Thing::from_meta(Meta::with_receiver(namespace_receiver));

  {
    let mut add = NamespaceBuilder::new(machine, &mut *infrastructure);
  }

  ObjectRef::new(infrastructure)
}
