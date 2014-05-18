//! Executions are like mutable, contained coroutines.
//!
//! From the [specification](http://ell.io/spec) itself:
//!
//! > Although they are similar to the more traditional "continuations" from
//! > programming-language theory, our executions are *not* static. One does not
//! > simply take an execution, and then have a handle to resume execution at
//! > the point it was taken indefinitely. When a particular execution-object is
//! > used to resume execution at that point, the object itself "moves forward"
//! > **with** the procedure's execution.

use std::io::IoResult;

use script::Script;
use object::{Object, ObjectRef};
use machine::Machine;

pub struct Execution {
  root:     Script,
  pristine: bool,
  pc:       ~[uint],
  stack:    ~[ObjectRef]
}

impl Execution {
  /// Creates a new Execution with the given Script as its root.
  pub fn new(root: Script) -> Execution {
    Execution {
      root:     root,
      pristine: true,
      pc:       ~[],
      stack:    ~[]
    }
  }

  /// Returns the "root" Script of the Execution, which the Execution's internal
  /// program counter ("pc") is based on.
  pub fn root<'a>(&'a self) -> &'a Script {
    &self.root
  }
}

impl Object for Execution {
  fn fmt_paws(&self, writer: &mut Writer, machine: &Machine) -> IoResult<()> {
    try!(write!(writer, "Execution \\{ root: "));

    try!(self.root.fmt_paws(writer, machine));

    try!(write!(writer, ", pristine: {}, pc: {}, stack: [",
      self.pristine, self.pc));

    let mut stack_iter = self.stack.iter().peekable();

    loop {
      match stack_iter.next() {
        Some(object_ref) => {
          try!(object_ref.deref().fmt_paws(writer, machine));

          if !stack_iter.is_empty() {
            try!(write!(writer, ", "));
          }
        },
        None => break
      }
    }

    try!(write!(writer, "] \\}"));

    Ok(())
  }
}
