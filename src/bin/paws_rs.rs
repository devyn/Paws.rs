extern crate paws;
extern crate native;
extern crate getopts;

use std::io;
use std::os;
use std::fmt;

use std::any::Any;

use std::io::fs::File;
use std::path::Path;

use std::sync::Future;

use getopts::{optopt, optflag, optflagmulti, getopts};
use getopts::{ArgumentMissing, UnrecognizedOption, OptionMissing};
use getopts::{OptionDuplicated, UnexpectedArgument};

use paws::cpaws;
use paws::machine::{Machine, Reactor};
use paws::object::execution::Execution;

#[start]
fn start(argc: int, argv: **u8) -> int {
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

    {cyan}-h, --help{reset}
      Displays this message.

                                               {white}~devyn ({blue}{underline}https://github.com/devyn{reset}{white}){reset}
",
  program   = os::args().get(0),
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

          optopt("R", "reactors", "", ""),

    optflagmulti("",  "no-stall", ""),
    optflagmulti("",     "stall", "")
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

  // Option: -R, --reactors COUNT
  let mut reactors = 1;

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

  // Now get input, either from stdin or a file
  let input:    String;
  let filename: String;

  if matches.free.len() > 1 {
    format_args!(argument_error,
      concat!("Error: must provide either a single file to run, or none, in",
              " which case input is taken from stdin.\n",
              "\n",
              "You provided {} non-option argument(s).\n"),
      matches.free.len());
    return

  } else if matches.free.is_empty() {
    input    = io::stdin().read_to_str().unwrap();
    filename = "<stdin>".to_string();

  } else {
    let path = Path::new(matches.free.get(0).as_slice());

    match File::open(&path).read_to_str() {
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

  if no_stall {
    machine.on_stall(proc(machine) {
      machine.stop();
    });
  }

  // Parse and stage input
  if !eval(&machine, input.as_slice(), filename.as_slice()) {
    return
  }

  // Spawn reactors
  let reactor_pool: Vec<Future<Result<(), Box<Any + Send>>>> =
    range(0, reactors).map(|_|
      Reactor::new(machine.clone()).spawn()
    ).collect();

  // Wait for reactors to finish
  for task in reactor_pool.move_iter() {
    task.unwrap().unwrap();
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
          os::args().get(0))).unwrap();

  os::set_exit_status(1);
}

fn eval(machine: &Machine, input: &str, filename: &str) -> bool {
  match cpaws::parse_nodes(input.as_slice(), filename) {
    Ok(nodes) => {
      // Compile an execution...
      let script        = cpaws::build_script(machine, nodes.as_slice());
      let execution_ref = machine.execution(script);

      // ...expose the system interface to it...
      machine.expose_system_to(
        &mut *execution_ref.lock().try_cast::<Execution>()
                                  .ok().unwrap());

      // and stage!
      machine.enqueue(execution_ref.clone(), execution_ref.clone());

      true
    }

    Err(message) => {
      format_args!(generic_error, "Parse error: {}", message);
      false
    }
  }
}
