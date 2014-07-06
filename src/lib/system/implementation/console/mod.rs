//! The console! For debugging and stuff.

#![allow(unused_variable)]

use object::*;
use object::thing::Thing;

use machine::*;

use util::namespace::*;

use std::io::stdio;

/// Generates an `implementation console` namespace object.
pub fn make(machine: &Machine) -> ObjectRef {
  let mut console =
    box Thing::from_meta(Meta::with_receiver(namespace_receiver));

  {
    let mut add = NamespaceBuilder::new(machine, &mut *console);

    add.oneshot(      "print",                   print                        );
    add.oneshot(      "inspect",                 inspect                      );
  }

  ObjectRef::new(console)
}

/// Prints a symbol to stdout. Doesn't return. Oneshot.
///
/// # Example
///
///     implementation console print "Hello, world!"
pub fn print(machine: &Machine, response: ObjectRef) -> Reaction {
  match response.symbol_ref() {
    Some(string) =>
      stdio::println(string.as_slice()),

    None => {
      warn!("tried to print[] a non-symbol");
    }
  }

  Yield
}

/// Debug-prints the given Object (`fmt_paws()`) to stdout. Doesn't return.
/// Oneshot.
///
/// # Example
///
///     implementation console inspect [locals]
pub fn inspect(machine: &Machine, response: ObjectRef) -> Reaction {
  let mut stdout = stdio::stdout();

  // FIXME: do something if these fail
  let _ = response.lock().fmt_paws(&mut stdout);
  let _ = stdout.write_char('\n');

  Yield
}
