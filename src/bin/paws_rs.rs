extern crate paws;
extern crate native;

use std::io;
use std::os;
use paws::cpaws;
use paws::machine::{Machine, Reactor};
use paws::object::execution::Execution;

#[start]
fn start(argc: int, argv: **u8) -> int {
  // Make sure we use the native (not green thread) runtime
  native::start(argc, argv, main)
}

fn main() {
  let input = io::stdin().read_to_str()
                .ok().expect("reading from stdin failed");

  match cpaws::parse_nodes(input.as_slice(), "<stdin>") {
    Ok(nodes) => {
      let machine       = Machine::new();
      let script        = cpaws::build_script(&machine, nodes.as_slice());
      let execution_ref = machine.execution(script);

      // Set the machine up...
      machine.expose_system_to(
        &mut *execution_ref.lock().try_cast::<Execution>()
                                  .ok().unwrap());

      machine.enqueue(execution_ref.clone(), execution_ref.clone());

      // And let's go!
      Reactor::new(machine).run();
    }

    Err(message) => {
      println!("Parse error: {}", message);
      os::set_exit_status(1);
    }
  }
}
