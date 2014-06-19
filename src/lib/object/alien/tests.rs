use object::alien::*;

use object::*;
use object::empty::Empty;
use object::symbol::*;

use machine::*;

use std::any::AnyRefExt;
use std::io::IoResult;

/// Concats any symbols it receives to its internal state.
#[deriving(Clone, Show, Eq, TotalEq)]
struct SymbolConcatenationRoutine {
  string: ~str
}

impl Routine for SymbolConcatenationRoutine {
  #[allow(unused_variable)]
  fn combine(&mut self, machine: &mut Machine, caller: ObjectRef,
             subject_meta: &mut Meta, message: ObjectRef) {
    let message_borrow = message.read();

    match message_borrow.as_any().as_ref::<Symbol>() {
      Some(symbol) =>
        self.string = self.string.to_owned().append(
                        symbol.name(&machine.symbol_map)),
      None => ()
    }
  }

  #[allow(unused_variable)]
  fn fmt_paws(&self, writer: &mut Writer, machine: &Machine) -> IoResult<()> {
    write!(writer, "{}", self)
  }
}

#[test]
fn symbol_concatenation_routine_alien() {
  let mut machine = Machine::new();

  let empty = ObjectRef::new(~Empty::new());
  let hello = ObjectRef::new(~machine.symbol("Hello, "));
  let world = ObjectRef::new(~machine.symbol("world!"));

  let mut alien = Alien::new(~SymbolConcatenationRoutine { string: ~"" });

  alien.combine(&mut machine, empty.clone(), hello.clone());
  alien.combine(&mut machine, empty.clone(), world.clone());

  assert!("Hello, world!" ==
            alien.routine.as_any().as_ref::<SymbolConcatenationRoutine>()
            .unwrap().string.as_slice());
}
