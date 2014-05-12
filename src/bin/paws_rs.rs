extern crate paws;

use std::io;
use std::os;

use paws::cpaws;
use paws::machine::Machine;

fn main() {
  let input = io::stdin().read_to_str()
                .ok().expect("reading from stdin failed");

  match cpaws::parse_nodes(input, "<stdin>") {
    Ok(nodes) => {
      let mut machine = Machine::new();

      let script = cpaws::build_script(&mut machine, nodes);

      script.fmt_paws(&mut io::stdout(), &machine)
        .ok().expect("fmt_paws did not succeed!");

      io::print("\n");
    }

    Err(message) => {
      println!("Parse error: {}", message);
      os::set_exit_status(1);
    }
  }
}
