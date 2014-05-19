extern crate paws;

use std::io;
use std::os;
use std::any::AnyMutRefExt;

use paws::cpaws;
use paws::machine::{Machine, Combination};
use paws::object::{Object, ObjectRef};
use paws::object::execution::Execution;

fn main() {
  let input = io::stdin().read_to_str()
                .ok().expect("reading from stdin failed");

  let mut stdout = io::stdout();

  match cpaws::parse_nodes(input, "<stdin>") {
    Ok(nodes) => {
      let mut machine = Machine::new();

      let test_symbol = ObjectRef::new(~machine.symbol("test"));

      let script = cpaws::build_script(&mut machine, nodes);

      let execution_ref =
        ObjectRef::new(~Execution::new(script));

      println!("Advancing script as Execution with \"test\" symbol.");

      loop {
        stdout.write_str("\n    ").unwrap();

        let mut maybe_combination: Option<Combination>;

        {
          let mut execution_ref_borrow = execution_ref.borrow_mut();

          let execution: &mut Execution =
            execution_ref_borrow.as_mut_any().as_mut().unwrap();

          execution.fmt_paws(&mut stdout, &machine)
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
                let subject_borrow = subject_ref.borrow();

                subject_borrow.fmt_paws(&mut stdout, &machine)
                  .ok().expect("fmt_paws did not succeed!");
              }
            }

            stdout.write_str(" <- ").unwrap();

            let message_borrow = combination.message.borrow();

            message_borrow.fmt_paws(&mut stdout, &machine)
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
