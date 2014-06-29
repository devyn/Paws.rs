//! Describes Paws "Scripts", which represent the flow of Objects to be sent in
//! sequence within an Execution.

use std::io::IoResult;
use object::{Object,ObjectRef};

/// The Nodes of a script are evaluated by combining them in series as the
/// `message`, with whatever the response of the last combination was as the
/// `subject`.
#[deriving(Clone, Eq, TotalEq)]
pub enum Node {
  /// Indicates that the given object should be combined as-is.
  ObjectNode(ObjectRef),

  /// Indicates that the nodes within should all be combined in series against
  /// the locals of the context (Execution) they are in, and the result should
  /// then be combined with the outer response.
  ExpressionNode(~[Node])
}

impl Node {
  /// Formats a Node for debugging.
  pub fn fmt_paws(&self, writer: &mut Writer) -> IoResult<()> {
    match self {
      &ObjectNode(ref object_ref) =>
        try!(object_ref.lock().fmt_paws(writer)),

      &ExpressionNode(ref nodes) => {
        try!(writer.write_str("Expression { "));
        try!(fmt_paws_nodes(nodes.as_slice(), writer));
        try!(writer.write_str(" }"));
      }
    }

    Ok(())
  }
}

/// Points to the root of a Script, which is an expression (in the same sense as
/// `ExpressionNode`) of many Nodes.
#[deriving(Clone, Eq, TotalEq)]
pub struct Script(~[Node]);

impl Script {
  /// Formats a Script for debugging.
  pub fn fmt_paws(&self, writer: &mut Writer) -> IoResult<()> {

    let &Script(ref nodes) = self;

    try!(writer.write_str("Script { "));
    try!(fmt_paws_nodes(nodes.as_slice(), writer));
    try!(writer.write_str(" }"));

    Ok(())
  }
}

fn fmt_paws_nodes(nodes: &[Node], writer: &mut Writer) -> IoResult<()> {

  let mut iterator = nodes.iter().peekable();

  loop {
    match iterator.next() {
      Some(node) => {
        try!(node.fmt_paws(writer));

        if !iterator.is_empty() {
          try!(writer.write_str(", "));
        }
      },
      None => break
    }
  }

  Ok(())
}
