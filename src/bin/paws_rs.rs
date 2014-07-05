extern crate paws;

use std::io;
use std::os;
use paws::cpaws;
use paws::machine::Machine;
use paws::object::execution::Execution;

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
      machine.run_reactor();
    }

    Err(message) => {
      println!("Parse error: {}", message);
      os::set_exit_status(1);
    }
  }
}
