//! The console! For debugging and stuff.

#![allow(unused_variable)]

use object::*;
use object::thing::Thing;
use object::alien::Alien;

use machine::*;

use util::namespace::*;

use std::any::*;
use std::io::stdio;

/// Generates an `implementation console` namespace object.
pub fn make(machine: &Machine) -> ObjectRef {
  let mut console =
    box Thing::from_meta(Meta::with_receiver(namespace_receiver));

  {
    let mut add = NamespaceBuilder::new(machine, &mut *console);

    add.factory(      "print",                   print                        );
  }

  ObjectRef::new(console)
}

/// Prints a symbol to stdout. Doesn't return.
///
/// # Queueing semantics
///
/// 1. Accept a symbol and print it.
///
/// # Example
///
///     implementation console print "Hello, world!"
pub fn print(machine: &Machine) -> Alien {
  #[deriving(Clone)]
  enum Completion {
    Incomplete,
    Complete
  }

  fn print_routine<'a>(
                   mut alien: TypedRefGuard<'a, Alien>,
                   machine:   &Machine,
                   response:  ObjectRef)
                   -> Reaction {

    match alien.data.as_mut::<Completion>() {
      Some(&Complete) =>
        return Yield,

      Some(completion) => {
        *completion = Complete;
      },

      None =>
        fail!("print_routine called on a non-print() Alien!")
    }

    alien.unlock();

    match response.symbol_ref() {
      Some(string) =>
        stdio::print(string.as_slice()),

      None => () // FIXME: should probably warn
    }

    Yield
  }

  Alien::new(print_routine, box Incomplete)
}
