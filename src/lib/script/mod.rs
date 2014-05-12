//! Describes Paws "Scripts", which represent the flow of Objects to be sent in
//! sequence within an Execution.

use std::io::IoResult;
use object::Object;
use machine::Machine;

/// A node can either be a single Object (`ObjectNode`) or a subexpression of
/// multiple Nodes to be executed in sequence (`ExpressionNode`).
pub enum Node {
  ObjectNode(~Object),
  ExpressionNode(~[Node])
}

impl Node {
  /// Formats a Node for debugging.
  pub fn fmt_paws(&self, writer: &mut Writer, machine: &Machine)
         -> IoResult<()> {

    match self {
      &ObjectNode(ref object) =>
        try!(object.fmt_paws(writer, machine)),

      &ExpressionNode(ref nodes) => {
        try!(writer.write_str("Expression { "));
        try!(fmt_paws_nodes(nodes.as_slice(), writer, machine));
        try!(writer.write_str(" }"));
      }
    }

    Ok(())
  }
}

/// Points to the root of a Script, which is an expression (in the same sense as
/// `ExpressionNode`) of many Nodes.
pub struct Script(~[Node]);

impl Script {
  /// Formats a Script for debugging.
  pub fn fmt_paws(&self, writer: &mut Writer, machine: &Machine)
         -> IoResult<()> {

    let &Script(ref nodes) = self;

    try!(writer.write_str("Script { "));
    try!(fmt_paws_nodes(nodes.as_slice(), writer, machine));
    try!(writer.write_str(" }"));

    Ok(())
  }
}

fn fmt_paws_nodes(nodes: &[Node], writer: &mut Writer, machine: &Machine)
   -> IoResult<()> {

  let mut iterator = nodes.iter().peekable();

  loop {
    match iterator.next() {
      Some(node) => {
        try!(node.fmt_paws(writer, machine));

        if !iterator.is_empty() {
          try!(writer.write_str(", "));
        }
      },
      None => break
    }
  }

  Ok(())
}
