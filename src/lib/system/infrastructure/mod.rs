//! Nucleus' standardized aliens for manipulating objects and the Machine.
//!
//! Because everything under the `infrastructure` namespace is standardized,
//! documentation will not be provided here for aliens, unless they have some
//! unusual Paws.rs-specific construction pattern.

#![allow(unused_variable)]
#![allow(missing_doc)]

use object::{ObjectRef, Meta};
use object::{ObjectReceiver, NativeReceiver};

use nuketype::{Thing, Alien};

use machine::{Machine, Reactor};

use util::namespace::NamespaceBuilder;
use util::clone;

pub mod label;
pub mod execution;

/// Generates an `infrastructure` namespace object.
pub fn make(machine: &Machine) -> ObjectRef {
  let mut infrastructure = Meta::new();

  {
    let mut add = NamespaceBuilder::new(machine, &mut infrastructure);

    add.factory(      "label",                   label::make                  );
    add.factory(      "execution",               execution::make              );

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

  Thing::tagged(infrastructure, "(infrastructure)")
}

pub fn empty(reactor: &mut Reactor, caller: ObjectRef, args: &[ObjectRef]) {

  reactor.stage(caller, Thing::empty());
}

pub fn get(reactor: &mut Reactor, caller: ObjectRef, args: &[ObjectRef]) {
  match args {
    [ref from, ref index] => {
      let index = match unsignedish(index) {
        Some(index) => index,
        None        => return 
      };

      match from.lock().meta().members.get(index) {
        Some(relationship) => reactor.stage(caller, relationship.to().clone()),
        None               => return
      }
    },
    _ => fail!("wrong number of arguments")
  }
}

pub fn set(reactor: &mut Reactor, caller: ObjectRef, args: &[ObjectRef]) {
  match args {
    [ref on, ref index, ref what] => {
      let index = match unsignedish(index) {
        Some(index) => index,
        None        => return
      };

      on.lock().meta_mut().members.set(index, what.clone());
    },
    _ => fail!("wrong number of arguments")
  }
}

pub fn cut(reactor: &mut Reactor, caller: ObjectRef, args: &[ObjectRef]) {
  match args {
    [ref from, ref index] => {
      let index = match unsignedish(index) {
        Some(index) => index,
        None        => return
      };

      match from.lock().meta_mut().members.delete(index) {
        Some(relationship) => reactor.stage(caller, relationship.to().clone()),
        None               => return
      }
    },
    _ => fail!("wrong number of arguments")
  }
}

pub fn affix(reactor: &mut Reactor, caller: ObjectRef, args: &[ObjectRef]) {
  match args {
    [ref onto, ref what] =>
      onto.lock().meta_mut().members.push(what.clone()),

    _ => fail!("wrong number of arguments")
  }
}

pub fn unaffix(reactor: &mut Reactor, caller: ObjectRef, args: &[ObjectRef]) {
  match args {
    [ref from] =>
      match from.lock().meta_mut().members.pop() {
        Some(relationship) => reactor.stage(caller, relationship.unwrap()),
        None               => return
      },
    _ => fail!("wrong number of arguments")
  }
}

pub fn prefix(reactor: &mut Reactor, caller: ObjectRef, args: &[ObjectRef]) {
  match args {
    [ref onto, ref what] =>
      onto.lock().meta_mut().members.insert(1, what.clone()),

    _ => fail!("wrong number of arguments")
  }
}

pub fn unprefix(reactor: &mut Reactor, caller: ObjectRef, args: &[ObjectRef]) {
  match args {
    [ref from] =>
      match from.lock().meta_mut().members.remove(1) {
        Some(relationship) => reactor.stage(caller, relationship.unwrap()),
        None               => return
      },
    _ => fail!("wrong number of arguments")
  }
}

pub fn length(reactor: &mut Reactor, caller: ObjectRef, args: &[ObjectRef]) {
  match args {
    [ref of] => {
      // We subtract 1 from the length because the noughty (#0) is not counted;
      // this is the length of the "data"-members.
      let length = of.lock().meta().members.len() as int - 1;

      let length_sym =
        reactor.machine().symbol(length.to_string().as_slice());

      reactor.stage(caller, length_sym);
    },
    _ => fail!("wrong number of arguments")
  }
}

pub fn find(reactor: &mut Reactor, caller: ObjectRef, args: &[ObjectRef]) {
  match args {
    [ref within, ref key] => {
      let result =
        match key.symbol_ref() {
          Some(sym) => reactor.cache().sym_lookup(within.clone(), sym.clone()),
          None      => within.lock().meta().members.lookup_pair(key)
        };

      match result {
        Some(value) => reactor.stage(caller, value),
        None        => return
      }
    },
    _ => fail!("wrong number of arguments")
  }
}

pub fn compare(reactor: &mut Reactor, caller: ObjectRef, args: &[ObjectRef]) {
  match args {
    [ref a, ref b] =>
      if a == b {
        reactor.stage(caller, a.clone())
      } else {
        return
      },
    _ => fail!("wrong number of arguments")
  }
}

pub fn clone(reactor: &mut Reactor, caller: ObjectRef, args: &[ObjectRef]) {
  match args {
    [ref original] =>
      reactor.stage(caller, clone::to_thing(original)),

    _ => fail!("wrong number of arguments")
  }
}

pub fn adopt(reactor: &mut Reactor, caller: ObjectRef, args: &[ObjectRef]) {
  match args {
    [ref from, ref onto] => {
      let members = from.lock().meta().members.clone();

      onto.lock().meta_mut().members = members;
    },
    _ => fail!("wrong number of arguments")
  }
}

pub fn receiver(reactor: &mut Reactor, caller: ObjectRef, args: &[ObjectRef]) {
  match args {
    [ref of] =>
      match of.lock().meta().receiver.clone() {
        ObjectReceiver(receiver) =>
          reactor.stage(caller, receiver),

        NativeReceiver(receiver) =>
          reactor.stage(caller, Alien::from_native_receiver(receiver)),
      },
    _ => fail!("wrong number of arguments")
  }
}

pub fn receive(reactor: &mut Reactor, caller: ObjectRef, args: &[ObjectRef]) {
  match args {
    [ref on, ref receiver] => {
      // TODO: see whether checking whether the 'receiver' is an Alien wrapping
      // a NativeReceiver and using that yields a performance advantage (it
      // should)
      on.lock().meta_mut().receiver = ObjectReceiver(receiver.clone());
    },
    _ => fail!("wrong number of arguments")
  }
}

pub fn own(reactor: &mut Reactor, caller: ObjectRef, args: &[ObjectRef]) {
  match args {
    [ref on, ref index] => {
      unsignedish(index).map(|index| {

        if !on.lock().meta_mut().members.own(index) {
          warn!("tried to own a nonexistent member #{} on {}",
            index, on);
        }

      });
    },
    _ => fail!("wrong number of arguments")
  }
}

pub fn disown(reactor: &mut Reactor, caller: ObjectRef, args: &[ObjectRef]) {
  match args {
    [ref on, ref index] => {
      unsignedish(index).map(|index| {

        if !on.lock().meta_mut().members.disown(index) {
          warn!("tried to disown a nonexistent member #{} on {}",
            index, on);
        }

      });
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
