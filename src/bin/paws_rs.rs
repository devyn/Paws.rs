extern crate paws;

use std::os::set_exit_status;

use paws::cpaws;

fn main() {
  let input = std::io::stdin().read_to_str()
                .ok().expect("reading from stdin failed");

  match cpaws::parse_nodes(input, "<stdin>") {
    Ok(nodes) => {
      println!("{:?}", nodes);
    }

    Err(message) => {
      println!("Parse error: {}", message);
      set_exit_status(1);
    }
  }
}
