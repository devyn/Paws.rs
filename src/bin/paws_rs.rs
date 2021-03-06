extern crate paws;
extern crate native;
extern crate getopts;

use std::io;
use std::os;
use std::fmt;

use std::io::fs::File;
use std::path::Path;

use getopts::{optopt, optflag, optflagmulti, getopts};
use getopts::{ArgumentMissing, UnrecognizedOption, OptionMissing};
use getopts::{OptionDuplicated, UnexpectedArgument};

use paws::cpaws;

use paws::machine::Machine;
use paws::machine::reactor::{Reactor, SerialReactor, ReactorPool};

use paws::nuketype::Execution;

use paws::specification::Suite;

use paws::interact::start as interact;

#[start]
fn start(argc: int, argv: *const *const u8) -> int {
  // Make sure we use the native (not green thread) runtime
  native::start(argc, argv, main)
}

fn help() {
  // FIXME: make color configurable

  print!("{white}Paws.rs                                            “it's less fancy, but faster”{reset}

  {bold}Usage: {reset}{cyan}{program} [options] [file.paws]{reset}

    By default, Paws.rs will consume a cPaws script from stdin and attempt to
    react it. If all goes well, it won't exit at all. If you provide a Paws
    file, that will be loaded instead.

  {bold}Options:{reset}

    {cyan}-i, --interact{reset}
      Starts a Paws.rs read-eval-print loop. All other options will be ignored.

    {cyan}--[no-]stall{reset}
      The default mode is {cyan}--stall{reset}, in which Paws.rs continues to run in an
      endless loop forever until you intentionally kill it, probably long after
      anything that would be put on the queue is still being produced.

      The non-standard {cyan}--no-stall{reset} mode allows Paws.rs to stop the machine
      automatically if it's impossible for any progress to be made. Useful for
      benchmarking or any kind of automation.

    {cyan}-R, --reactors COUNT{reset}
      Tells Paws.rs how many reactors to spawn in parallel. The default is 1
      reactor (no parallelism).

      Note: this is highly experimental, and is more likely to result in a
      decrease in performance than an increase.

    {cyan}--spec{reset}
      Runs Paws.rs in specification mode, allowing it to run tests provided by
      the Paws Rulebook. The output conforms to the Test Anything Protocol.

      This option implies {cyan}--no-stall{reset}.

    {cyan}-h, --help{reset}
      Displays this message.

                                               {white}~devyn ({blue}{underline}https://github.com/devyn{reset}{white}){reset}
",
  program   = os::args()[0],
  reset     = "\x1b[0m",
  bold      = "\x1b[1m",
  underline = "\x1b[4m",
  blue      = "\x1b[34m",
  cyan      = "\x1b[36m",
  white     = "\x1b[37m");
}

fn main() {
  let args: Vec<String> = os::args();

  // Descriptions are found in help(), not here.
  let opts = [
         optflag("h",     "help", ""),

         optflag("i", "interact", ""),

          optopt("R", "reactors", "", ""),

    optflagmulti("",  "no-stall", ""),
    optflagmulti("",     "stall", ""),

         optflag("",      "spec", "")
  ];

  let matches = match getopts(args.tail(), opts) {
    Ok(m)  => m,
    Err(f) => {
      handle_getopts_error(f);
      return
    }
  };

  // Flag: -h, --help
  if matches.opt_present("h") {
    help();
    return
  }

  // Flag: -i, --interact
  if matches.opt_present("i") {
    interact();
    return
  }

  // Option: -R, --reactors COUNT
  let mut reactors: int = 1;

  match matches.opt_str("reactors") {
    Some(n) =>
      match from_str::<int>(n.as_slice()) {
        Some(n) if n <= 0 => {
          format_args!(argument_error,
            "Error: you must start at least one reactor!");
          return
        },

        None => {
          format_args!(argument_error,
            concat!("Error: --reactors should be given a number greater",
                    " than zero."));
          return
        },

        Some(n) =>
          reactors = n
      },
    None => ()
  }

  // Flags: --no-stall, --stall
  let mut no_stall = false;

  if matches.opt_count("no-stall") > matches.opt_count("stall") {
    no_stall = true;
  }

  // Flag: --spec
  let spec_ = matches.opt_present("spec");

  // Now get input, either from stdin or files
  let input;
  let filename;

  if matches.free.len() > 1 {
    format_args!(argument_error,
      concat!("Error: must provide either a single file to run, or none, in",
              " which case input is taken from stdin.\n",
              "\n",
              "You provided {} non-option argument(s).\n"),
      matches.free.len());
    return

  } else if matches.free.is_empty() {
    input    = io::stdin().read_to_string().unwrap();
    filename = "<stdin>".to_string();

  } else {
    let path = Path::new(matches.free[0].as_slice());

    match File::open(&path).read_to_string() {
      Ok(string) => {
        input    = string;
        filename = format!("{}", path.display());
      },

      Err(e) => {
        format_args!(generic_error, "Error: {}", e);
        return
      }
    }
  }

  // Set up machine as requested
  let machine = Machine::new();

  let start = proc (reactor: &mut Reactor) {
    if spec_ {
      // Parse and stage input (in spec mode)
      spec(reactor, input.as_slice(), filename.as_slice())
    } else {
      // Parse and stage input
      if !eval(reactor, input.as_slice(), filename.as_slice()) {
        return false
      }

      if no_stall {
        reactor.on_stall(proc(reactor) {
          reactor.stop();
        });
      }

      true
    }
  };

  // Spawn reactors
  if reactors == 1 {
    let mut reactor = SerialReactor::new(machine);

    if !start(&mut reactor) { return }

    reactor.run()
  } else {
    let mut pool = ReactorPool::spawn(machine, reactors as uint);

    pool.on_reactor(proc (reactor) {
      let ok = start(&mut *reactor);

      if !ok {
        reactor.stop()
      }
    });

    pool.wait()
  }
}

fn handle_getopts_error(error: getopts::Fail_) {
  match error {
    ArgumentMissing(arg) =>
      format_args!(argument_error,
                   concat!("Error: '{}' requires an argument, but none",
                           " was found.\n"),
                   arg),

    UnrecognizedOption(arg) =>
      format_args!(argument_error,
                   "Error: unrecognized option '{}'.\n", arg),

    OptionMissing(arg) =>
      format_args!(argument_error,
                   concat!("Error: '{}' is a required option, but was not",
                           " found.\n"),
                   arg),

    OptionDuplicated(arg) =>
      format_args!(argument_error,
                   "Error: '{}' must not appear more than once.\n", arg),

    UnexpectedArgument(arg) =>
      format_args!(argument_error,
                   "Error: unrecognized option '{}'.\n", arg)
  }
}

fn generic_error(args: &fmt::Arguments) {
  let mut stderr = io::stderr();

  stderr.write_str(fmt::format(args).as_slice()).unwrap();

  os::set_exit_status(1);
}

fn argument_error(args: &fmt::Arguments) {
  let mut stderr = io::stderr();

  stderr.write_str(fmt::format(args).as_slice()).unwrap();

  (write!(stderr,
          concat!("\n",
                  "    $ {} --help\n",
                  "\n",
                  "might help you figure out what went wrong.\n"),
          os::args()[0])).unwrap();

  os::set_exit_status(1);
}

fn eval(reactor: &mut Reactor, input: &str, filename: &str) -> bool {
  match cpaws::parse_nodes(input.as_slice(), filename) {
    Ok(nodes) => {
      // Compile an execution...
      let script        = cpaws::build_script(reactor.machine(),
                                              nodes.as_slice());
      let execution_ref = Execution::create(reactor.machine(), script);

      // ...expose the system interface to it...
      reactor.machine().expose_system_to(&execution_ref);

      // and stage!
      reactor.stage(execution_ref.clone(), execution_ref.clone());

      true
    }

    Err(message) => {
      format_args!(generic_error, "Parse error: {}", message);
      false
    }
  }
}

fn spec(reactor: &mut Reactor, input: &str, filename: &str) -> bool {
  match cpaws::parse_nodes(input.as_slice(), filename) {
    Ok(nodes) => {
      let suite = Suite::new();

      // Compile an execution...
      let script        = cpaws::build_script(reactor.machine(),
                                              nodes.as_slice());
      let execution_ref = Execution::create(reactor.machine(), script);

      // ...expose the system interface to it...
      reactor.machine().expose_system_to(&execution_ref);

      // ...expose the specification interface to it...
      suite.expose_to(&execution_ref, reactor.machine());

      // and stage!
      reactor.stage(execution_ref.clone(), execution_ref.clone());

      // Then, run the suite once that stalls.
      reactor.on_stall(proc (reactor) {
        suite.run(reactor);
      });

      true
    }

    Err(message) => {
      format_args!(generic_error, "Parse error: {}", message);
      false
    }
  }
}
