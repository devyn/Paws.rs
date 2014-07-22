//! Procedures specific to `Symbol`s.
//!
//! **FIXME:** Nucleus calls them labels.

use object::*;
use object::thing::Thing;
use object::symbol::Symbol;

use machine::*;

use util::namespace::*;

/// Generates an `infrastructure label` namespace object.
pub fn make(machine: &Machine) -> ObjectRef {
  let mut label = box Thing::new();

  {
    let mut add = NamespaceBuilder::new(machine, &mut *label);

    add.call_pattern( "clone",                   clone, 1                     );
    add.call_pattern( "compare",                 compare, 2                   );
    add.call_pattern( "explode",                 explode, 1                   );
  }

  ObjectRef::new_with_tag(label, "(infra. label)")
}

pub fn clone(reactor: &mut Reactor, caller: ObjectRef, args: &[ObjectRef]) {
  match args {
    [ref original] =>
      match original.lock().try_cast::<Symbol>() {
        Ok(symbol) =>
          reactor.stage(caller, ObjectRef::new_symbol(
                                  box symbol.deref().clone())),

        Err(_) =>
          warn!("tried to label clone[] {}, which is not a Symbol",
            original)
      },
    _ => fail!("wrong number of arguments")
  }
}

pub fn compare(reactor: &mut Reactor, caller: ObjectRef, args: &[ObjectRef]) {
  match args {
    [ref a, ref b] =>
      if a.eq_as_symbol(b) {
        reactor.stage(caller, a.clone())
      } else {
        return
      },
    _ => fail!("wrong number of arguments")
  }
}

pub fn explode(reactor: &mut Reactor, caller: ObjectRef, args: &[ObjectRef]) {
  match args {
    [ref symbol] =>
      match symbol.symbol_ref() {
        Some(string) => {
          let mut meta = Meta::new();

          let str_slice = string.as_slice();

          for (index, _) in str_slice.char_indices() {
            let char_str =
              str_slice.slice_from(index).slice_chars(0, 1);

            meta.members.push(reactor.machine().symbol(char_str));
          }

          reactor.stage(caller, ObjectRef::new(box Thing::from_meta(meta)))
        },
        None =>
          warn!("tried to label explode[] {}, which is not a Symbol",
            symbol)
      },
    _ => fail!("wrong number of arguments")
  }
}
