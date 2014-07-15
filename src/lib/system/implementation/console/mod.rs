//! The console! For debugging and stuff.

#![allow(unused_variable)]

use object::*;
use object::thing::Thing;

use machine::*;

use util::namespace::*;

use std::io::stdio;

use term;
use term::Terminal;

/// Generates an `implementation console` namespace object.
pub fn make(machine: &Machine) -> ObjectRef {
  let mut console = box Thing::new();

  {
    let mut add = NamespaceBuilder::new(machine, &mut *console);

    add.oneshot(      "print",                   print                        );
    add.oneshot(      "show",                    show                         );
    add.oneshot(      "inspect",                 inspect                      );
    add.call_pattern( "trace",                   trace, 1                     );
  }

  ObjectRef::new_with_tag(console, "(impl. console)")
}

/// Prints a Symbol to stdout. Doesn't return. Oneshot.
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

/// Debug-prints the given Object's **reference** to stdout. Doesn't return.
/// Oneshot.
///
/// The difference between this and `inspect()` is that this contains *a lot*
/// less information. It doesn't look deeply into the Object itself; rather, it
/// prints either the address or the symbol of the reference, and the tag, if
/// debugging is enabled and one is present.
///
/// # Example
///
///     implementation console show []
pub fn show(machine: &Machine, response: ObjectRef) -> Reaction {
  println!("{}", response);
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

/// Prints a message to the console, including information about the caller.
/// Returns the message.
///
/// The message can be any object. If it is a Symbol, it is printed verbatim;
/// else, `fmt_paws()` is used, like `inspect()`.
///
/// # Call-pattern arguments
///
/// 1. The message to print.
pub fn trace(machine: &Machine, caller: ObjectRef, args: &[ObjectRef])
             -> Reaction {

  let mut terminal = term::stdout().expect("terminal could not be opened");

  // FIXME: do something if these fail
  let _ = terminal.fg(term::color::CYAN);
  let _ = write!(terminal, "Trace {}:", caller);
  let _ = terminal.fg(term::color::WHITE);
  let _ = terminal.write_char(' ');

  let _ = match args {
    [ref symbol] => match symbol.symbol_ref() {
      Some(string) => write!(terminal, "{:s}", string.as_slice()),
      None         => symbol.lock().fmt_paws(terminal.get_mut())
    },
    _ => fail!("wrong number of arguments")
  };

  let _ = terminal.reset();
  let _ = terminal.write_char('\n');

  React(caller, args[0].clone())
}
