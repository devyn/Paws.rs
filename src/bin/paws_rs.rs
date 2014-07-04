extern crate paws;

use std::io;
use std::os;
use paws::cpaws;
use paws::machine::{Machine, Combination};
use paws::object::{Object, ObjectRef};
use paws::object::execution::Execution;

fn main() {
  let input = io::stdin().read_to_str()
                .ok().expect("reading from stdin failed");

  let mut stdout = io::stdout();

  match cpaws::parse_nodes(input.as_slice(), "<stdin>") {
    Ok(nodes) => {
      let machine = Machine::new();

      let test_symbol = machine.symbol("test");

      let script = cpaws::build_script(&machine, nodes.as_slice());

      let execution_ref =
        ObjectRef::new(box Execution::new(script));

      println!("Advancing script as Execution with \"test\" symbol.");

      loop {
        stdout.write_str("\n    ").unwrap();

        let mut maybe_combination: Option<Combination>;

        {
          let mut execution = execution_ref.lock().try_cast::<Execution>()
                                .ok().unwrap();

          execution.fmt_paws(&mut stdout)
            .ok().expect("fmt_paws did not succeed!");

          stdout.write_str("\n\n").unwrap();

          maybe_combination =
            execution.advance(execution_ref.clone(), test_symbol.clone());
        }

        match maybe_combination {
          Some(combination) => {
            stdout.write_str("(").unwrap();

            match combination.subject {
              None => stdout.write_str("#<locals>").unwrap(),
              Some(ref subject_ref) => {
                let subject = subject_ref.lock();

                subject.deref().fmt_paws(&mut stdout)
                  .ok().expect("fmt_paws did not succeed!");
              }
            }

            stdout.write_str(" <- ").unwrap();

            let message = combination.message.lock();

            message.deref().fmt_paws(&mut stdout)
              .ok().expect("fmt_paws did not succeed!");

            stdout.write_str(")").unwrap();
          },
          None => break
        }
      }
    }

    Err(message) => {
      println!("Parse error: {}", message);
      os::set_exit_status(1);
    }
  }
}
