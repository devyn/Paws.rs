//! Locals make up the in-`Execution` data storage objects.

use object::{ObjectRef, Meta};
use object::{NativeReceiver, Params};

use nuketype::Nuketype;

use machine::Reactor;

use std::io::IoResult;

#[cfg(test)]
mod tests;

/// Contains a 'name' symbol (usually representing the string "locals") to
/// which, if its receiver (`locals_receiver`) is invoked with it, it will
/// respond with the `Locals` object itself.
///
/// Otherwise acts more or less like a `Thing`.
#[deriving(Clone)]
pub struct Locals {
  name: ObjectRef
}

impl Locals {
  /// Creates a new `Locals` with the given name.
  pub fn new(name: ObjectRef) -> Locals {
    Locals {
      name: name
    }
  }

  /// Boxes a new `Locals` with the given name and metadata.
  ///
  /// The metadata's receiver will be overridden and set to `locals_receiver`.
  pub fn create(name: ObjectRef, mut meta: Meta) -> ObjectRef {
    meta.receiver = NativeReceiver(locals_receiver);
    
    ObjectRef::store(box Locals::new(name), meta)
  }

  /// Boxes a new `Locals` with the given name and empty metadata, with
  /// `locals_receiver` as the receiver.
  pub fn empty(name: ObjectRef) -> ObjectRef {
    Locals::create(name, Meta::new())
  }
}

impl Nuketype for Locals {
  fn fmt_paws(&self, writer: &mut Writer) -> IoResult<()> {
    write!(writer, "Locals")
  }
}

/// Returns the `subject` if the `message` is the `subject`'s name. Otherwise,
/// compares pair-wise, like `lookup_receiver`.
///
/// Behaves just like `lookup_receiver` if the subject is not of the `Locals`
/// nuketype.
pub fn locals_receiver(reactor: &mut Reactor, params: Params) {
  let lookup_result = {
    match params.subject.lock().try_cast::<Locals>() {
      Ok(subject) =>
        if params.message.eq_as_symbol(&subject.deref().name) {
          Some(params.subject.clone())
        } else {
          subject.meta().members.lookup_pair(&params.message)
        },
      Err(subject) =>
        subject.meta().members.lookup_pair(&params.message)
    }
  };

  debug!("{} <locals_receiver> {} => {}",
    params.subject, params.message, lookup_result);

  match lookup_result {
    Some(value) =>
      reactor.stage(params.caller.clone(), value),
    None =>
      return
  }
}
