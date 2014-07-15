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

pub fn clone(machine: &Machine, caller: ObjectRef, args: &[ObjectRef])
             -> Reaction {
  match args {
    [ref original] =>
      match original.lock().try_cast::<Symbol>() {
        Ok(symbol) =>
          React(caller, ObjectRef::new_symbol(box symbol.deref().clone())),

        Err(_) => {
          warn!("tried to label clone[] {}, which is not a Symbol",
            original);

          Yield
        }
      },
    _ => fail!("wrong number of arguments")
  }
}

pub fn compare(machine: &Machine, caller: ObjectRef, args: &[ObjectRef])
               -> Reaction {
  match args {
    [ref a, ref b] =>
      if a.eq_as_symbol(b) {
        React(caller, a.clone())
      } else {
        Yield
      },
    _ => fail!("wrong number of arguments")
  }
}

pub fn explode(machine: &Machine, caller: ObjectRef, args: &[ObjectRef])
               -> Reaction {
  match args {
    [ref symbol] =>
      match symbol.symbol_ref() {
        Some(string) => {
          let mut meta = Meta::new();

          let str_slice = string.as_slice();

          for (index, _) in str_slice.char_indices() {
            meta.members.push(
              machine.symbol(str_slice.slice_from(index).slice_chars(0, 1)));
          }

          React(caller, ObjectRef::new(box Thing::from_meta(meta)))
        },
        None => {
          warn!("tried to label explode[] {}, which is not a Symbol",
            symbol);

          Yield
        }
      },
    _ => fail!("wrong number of arguments")
  }
}
