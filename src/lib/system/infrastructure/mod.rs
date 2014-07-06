//! Nucleus' standardized aliens for manipulating objects and the Machine.
//!
//! Because everything under the `infrastructure` namespace is standardized,
//! documentation will not be provided here for aliens, unless they have some
//! unusual Paws.rs-specific construction pattern.

#![allow(unused_variable)]
#![allow(missing_doc)]

use object::*;
use object::thing::Thing;
use object::alien::Alien;

use machine::*;

use util::namespace::*;

pub mod label;
pub mod execution;

/// Generates an `infrastructure` namespace object.
pub fn make(machine: &Machine) -> ObjectRef {
  let mut infrastructure =
    box Thing::from_meta(Meta::with_receiver(namespace_receiver));

  {
    let mut add = NamespaceBuilder::new(machine, &mut *infrastructure);

    add.namespace(    "label",                   label::make                  );
    add.namespace(    "execution",               execution::make              );

    add.call_pattern( "empty",                   empty, 0                     );

    add.call_pattern( "get",                     get, 2                       );
    add.call_pattern( "set",                     set, 3                       );
    add.call_pattern( "cut",                     cut, 2                       );

    add.call_pattern( "affix",                   affix, 2                     );
    add.call_pattern( "unaffix",                 unaffix, 1                   );
    add.call_pattern( "prefix",                  prefix, 2                    );
    add.call_pattern( "unprefix",                unprefix, 2                  );

    add.call_pattern( "length",                  length, 1                    );

    add.call_pattern( "find",                    find, 2                      );

    add.call_pattern( "compare",                 compare, 2                   );
    add.call_pattern( "clone",                   clone, 1                     );
    add.call_pattern( "adopt",                   adopt, 2                     );

    add.call_pattern( "receiver",                receiver, 1                  );
    add.call_pattern( "receive",                 receive, 2                   );

    add.call_pattern( "own",                     own, 2                       );
    add.call_pattern( "disown",                  disown, 2                    );
  }

  ObjectRef::new(infrastructure).tag("(infrastructure)")
}

pub fn empty(machine: &Machine, caller: ObjectRef, args: &[ObjectRef])
             -> Reaction {

  React(caller, ObjectRef::new(box Thing::new()))
}

pub fn get(machine: &Machine, caller: ObjectRef, args: &[ObjectRef])
           -> Reaction {
  match args {
    [ref from, ref index] => {
      let index = match unsignedish(index) {
        Some(index) => index,
        None        => return Yield
      };

      match from.lock().meta().members.get(index) {
        Some(relationship) => React(caller, relationship.to().clone()),
        None               => Yield
      }
    },
    _ => fail!("wrong number of arguments")
  }
}

pub fn set(machine: &Machine, caller: ObjectRef, args: &[ObjectRef])
           -> Reaction {
  match args {
    [ref on, ref index, ref what] => {
      let index = match unsignedish(index) {
        Some(index) => index,
        None        => return Yield
      };

      on.lock().meta_mut().members.set(index, what.clone());
      Yield
    },
    _ => fail!("wrong number of arguments")
  }
}

pub fn cut(machine: &Machine, caller: ObjectRef, args: &[ObjectRef])
           -> Reaction {
  match args {
    [ref from, ref index] => {
      let index = match unsignedish(index) {
        Some(index) => index,
        None        => return Yield
      };

      match from.lock().meta_mut().members.delete(index) {
        Some(relationship) => React(caller, relationship.to().clone()),
        None               => Yield
      }
    },
    _ => fail!("wrong number of arguments")
  }
}

pub fn affix(machine: &Machine, caller: ObjectRef, args: &[ObjectRef])
             -> Reaction {
  match args {
    [ref onto, ref what] => {
      onto.lock().meta_mut().members.push(what.clone());
      Yield
    },
    _ => fail!("wrong number of arguments")
  }
}

pub fn unaffix(machine: &Machine, caller: ObjectRef, args: &[ObjectRef])
               -> Reaction {
  match args {
    [ref from] =>
      match from.lock().meta_mut().members.pop() {
        Some(relationship) => React(caller, relationship.unwrap()),
        None               => Yield
      },
    _ => fail!("wrong number of arguments")
  }
}

pub fn prefix(machine: &Machine, caller: ObjectRef, args: &[ObjectRef])
              -> Reaction {
  match args {
    [ref onto, ref what] => {
      onto.lock().meta_mut().members.unshift(what.clone());
      Yield
    },
    _ => fail!("wrong number of arguments")
  }
}

pub fn unprefix(machine: &Machine, caller: ObjectRef, args: &[ObjectRef])
                -> Reaction {
  match args {
    [ref from] =>
      match from.lock().meta_mut().members.shift() {
        Some(relationship) => React(caller, relationship.unwrap()),
        None               => Yield
      },
    _ => fail!("wrong number of arguments")
  }
}

pub fn length(machine: &Machine, caller: ObjectRef, args: &[ObjectRef])
              -> Reaction {
  match args {
    [ref of] => {
      let length = of.lock().meta().members.len() as int;

      // We subtract 1 from the length because the noughty (#0) is not counted;
      // this is the length of the "data"-members.
      React(caller, machine.symbol((length - 1).to_str().as_slice()))
    },
    _ => fail!("wrong number of arguments")
  }
}

pub fn find(machine: &Machine, caller: ObjectRef, args: &[ObjectRef])
            -> Reaction {
  match args {
    [ref within, ref key] =>
      match within.lock().meta().members.lookup_pair(key) {
        Some(value) => React(caller, value),
        None        => Yield
      },
    _ => fail!("wrong number of arguments")
  }
}

pub fn compare(machine: &Machine, caller: ObjectRef, args: &[ObjectRef])
               -> Reaction {
  match args {
    [ref a, ref b] =>
      if a == b {
        React(caller, a.clone())
      } else {
        Yield
      },
    _ => fail!("wrong number of arguments")
  }
}

pub fn clone(machine: &Machine, caller: ObjectRef, args: &[ObjectRef])
             -> Reaction {
  match args {
    [ref original] => {
      let mut meta = Meta::new();

      meta.members = original.lock().meta().members.clone();

      React(caller, ObjectRef::new(box Thing::from_meta(meta)))
    },
    _ => fail!("wrong number of arguments")
  }
}

pub fn adopt(machine: &Machine, caller: ObjectRef, args: &[ObjectRef])
             -> Reaction {
  match args {
    [ref from, ref onto] => {
      let members = from.lock().meta().members.clone();

      onto.lock().meta_mut().members = members;

      Yield
    },
    _ => fail!("wrong number of arguments")
  }
}

pub fn receiver(machine: &Machine, caller: ObjectRef, args: &[ObjectRef])
                -> Reaction {
  match args {
    [ref of] =>
      match of.lock().meta().receiver.clone() {
        ObjectReceiver(receiver) =>
          React(caller, receiver),

        NativeReceiver(receiver) =>
          React(caller, ObjectRef::new(box
                          Alien::from_native_receiver(receiver)))
      },
    _ => fail!("wrong number of arguments")
  }
}

pub fn receive(machine: &Machine, caller: ObjectRef, args: &[ObjectRef])
               -> Reaction {
  match args {
    [ref on, ref receiver] => {
      // TODO: see whether checking whether the 'receiver' is an Alien wrapping
      // a NativeReceiver and using that yields a performance advantage (it
      // should)
      on.lock().meta_mut().receiver = ObjectReceiver(receiver.clone());

      Yield
    },
    _ => fail!("wrong number of arguments")
  }
}

pub fn own(machine: &Machine, caller: ObjectRef, args: &[ObjectRef])
           -> Reaction {
  match args {
    [ref on, ref index] => {
      unsignedish(index).map(|index| {

        if !on.lock().meta_mut().members.own(index) {
          warn!("tried to own a nonexistent member #{} on {}",
            index, on);
        }

      });

      Yield
    },
    _ => fail!("wrong number of arguments")
  }
}

pub fn disown(machine: &Machine, caller: ObjectRef, args: &[ObjectRef])
              -> Reaction {
  match args {
    [ref on, ref index] => {
      unsignedish(index).map(|index| {

        if !on.lock().meta_mut().members.disown(index) {
          warn!("tried to disown a nonexistent member #{} on {}",
            index, on);
        }

      });

      Yield
    },
    _ => fail!("wrong number of arguments")
  }
}

// FIXME when ELLIOTTCABLE decides what he wants to do about numbers.
fn unsignedish(symbol: &ObjectRef) -> Option<uint> {
  symbol.symbol_ref().and_then(|string|
    from_str::<uint>(string.as_slice())
  )
}
