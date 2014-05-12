//! Paws objects, and a trait that they all share

use std::io::IoResult;
use machine::Machine;

pub mod symbol;

/// The interface that all Paws Objects must implement.
pub trait Object {
  /// Formats a Paws Object for debugging purposes.
  ///
  /// **TODO:** `machine` should really be moved out of here if possible. It's
  /// only required for looking symbols up in the machine's `SymbolMap`.
  fn fmt_paws(&self, writer: &mut Writer, machine: &Machine) -> IoResult<()>;
}
