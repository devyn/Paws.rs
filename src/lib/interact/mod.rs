//! Tools for configuring and starting a Paws read-eval-print loop.

use script::*;

use cpaws;

use machine::Machine;
use machine::reactor::{Reactor, SerialReactor};

use object::{ObjectRef, TypedRefGuard};

use nuketype::{Alien, Execution};

use term::{mod, Terminal};

use std::any::AnyRefExt;
use std::io::{mod, IoResult};

/// Start a new REPL in the default environment. This consists of:
///
/// * A new machine.
/// * A new serial reactor.
/// * Execution template containing `locals` with:
///   * `paws::system` (`infrastructure` and `implementation`)
pub fn start() {
  let machine  = Machine::new();
  let template = Execution::create(&machine, Script(vec![]));

  machine.expose_system_to(&template);

  start_with(machine, proc(machine) SerialReactor::new(machine), template);
}

/// Start a new REPL in a custom environment.
///
/// Only serial reactors are supported at the moment. The `template`'s metadata
/// is used to create the first Execution.
pub fn start_with(    machine:      Machine,
                      make_reactor: proc (Machine): Send -> SerialReactor,
                  mut template:     ObjectRef) {

  let mut stdout = term::stdout().expect("failed to open stdout!");

  let mut line: u64 = 1;

  fn prompt<T: Writer>(line: u64, stdout: &mut Terminal<T>) -> IoResult<()> {

    try!(stdout.fg(term::color::GREEN));

    try!(write!(stdout, "{:4u} ← ", line));

    try!(stdout.reset());

    stdout.flush()
  }

  fn error<T: Writer>(message: &str, stdout: &mut Terminal<T>) -> IoResult<()> {

    try!(stdout.fg(term::color::RED));

    try!(stdout.write_str(message));
    try!(stdout.write_char('\n'));
    try!(stdout.write_char('\n'));

    stdout.reset()
  }

  let (reactor_tx, reactor_rx) = sync_channel(0);

  let machine2 = machine.clone();

  spawn(proc() reactor_loop(make_reactor(machine2), reactor_rx));

  prompt(line, stdout).unwrap();

  for line_str in io::stdin().lines() {
    let mut line_str = line_str.unwrap();

    line_str.pop_char(); // drop the \n

    if !line_str.is_empty() {
      match parse(&machine, line, line_str.as_slice()) {
        Ok(execution) => {
          template = ObjectRef::store_with_tag(
            box execution, template.lock().meta().clone(),
            format!("interact {:u}", line));

          reactor_tx.send(Some(template.clone()));
          reactor_tx.send(None); // wait for the reactor to be ready
        },
        Err(message) => error(message.as_slice(), stdout).unwrap()
      }

      line += 1
    }

    prompt(line, stdout).unwrap();
  }
}

fn reactor_loop(mut reactor:  SerialReactor,
                    rx:       Receiver<Option<ObjectRef>>) {
  loop {
    let mut break_after = 1000u;

    while break_after > 1 && reactor.step() {
      break_after -= 1;
    }

    if reactor.step() {
      match rx.try_recv().ok() {
        Some(Some(execution)) =>
          reactor.stage(execution.clone(), execution),

        _ => ()
      }
    } else {
      reactor.stall();

      if !reactor.step() {
        match rx.recv_opt().ok() {
          Some(Some(execution)) =>
            reactor.stage(execution.clone(), execution),

          Some(None) => (),

          None => break
        }
      }
    }
  }
}

fn parse(machine:  &Machine,
         line:     u64,
         line_str: &str)
         -> Result<Execution, String> {

  cpaws::parse_nodes(line_str, (format!("<interact {:u}>", line)).as_slice())
    .map(|nodes| {
      let Script(mut instructions) =
        cpaws::build_script(machine, nodes.as_slice());

      // Inject a little wrapper into the Script in order to print out the
      // result.
      //
      // A normal pristine script looks like this:
      //
      //     [Discard, PushLocals, ...]
      //
      // We modify that to:
      //
      //     [Discard, Push(print), PushLocals, ..., Combine]

      assert!(instructions[0] == Discard);

      instructions.insert(1, Push(print(line)));

      instructions.push(Combine);

      Execution::new(Script(instructions))
    })
}

fn print(line: u64) -> ObjectRef {
  #[deriving(Clone)]
  struct PrintData(u64);

  fn routine<'a>(
             mut alien: TypedRefGuard<'a, Alien>,
             _reactor:  &mut Reactor,
             response:  ObjectRef) {

    let &PrintData(line) = alien.data.downcast_ref::<PrintData>().unwrap();

    let mut stdout = term::stdout().expect("failed to open stdout!");

    stdout.fg(term::color::BRIGHT_YELLOW).unwrap();

    (write!(stdout, "{:4u} → ", line)).unwrap();

    stdout.fg(term::color::WHITE).unwrap();

    (write!(stdout, "{}\n\n", response)).unwrap();

    stdout.reset().unwrap();
  }

  Alien::create(format!("interact {} print", line),
                routine, box PrintData(line))
}
