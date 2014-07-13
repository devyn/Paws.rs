//! Describes Paws "Scripts", which represent the combinations to be carried out
//! within an Execution, given a stack.

use object::ObjectRef;

/// Represents an instruction to be carried out over the Execution's stack.
#[deriving(Clone, PartialEq, Eq, Show)]
pub enum Instruction {
  /// Push the Execution's locals onto the stack.
  PushLocals,

  /// Push the Execution itself onto the stack.
  PushSelf,

  /// Push an object onto the stack.
  Push(ObjectRef),

  /// Pop off the stack as the subject, take the response as the message,
  /// combine, and unstage.
  Combine,

  /// Drop the top item off the stack, if there was one.
  Discard
}

/// A script is a sequence of instructions.
#[deriving(Clone, PartialEq, Eq, Show)]
pub struct Script(pub Vec<Instruction>);
