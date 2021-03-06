//! Paws Rulebook-compliant specification interface.
//!
//! The output conforms to the
//! [Test Anything Protocol](http://testanything.org/).

use object::{ObjectRef, TypedRefGuard, Meta};

use nuketype::{Thing, Alien};

use machine::{Machine, Reactor};

use std::any::AnyMutRefExt;
use std::sync::{Arc, Mutex};

/// Represents a test suite, containing rules that are added via the
/// specification interface (see `expose_to()`).
#[deriving(Clone)]
pub struct Suite {
  rules: Arc<Mutex<Vec<Rule>>>
}

impl Suite {
  /// Construct a new Suite.
  pub fn new() -> Suite {
    Suite {
      rules: Arc::new(Mutex::new(Vec::new()))
    }
  }

  /// Expose the specification interface to the given Execution's locals.
  pub fn expose_to(&self, execution: &ObjectRef, machine: &Machine) {
    let mut specification = Meta::new();

    specification.members
      .push_pair(machine.symbol("rule"), self.rule_alien());

    let specification  = Thing::tagged(specification, 
                                       "(specification)");

    let     locals_ref = execution.lock().meta().members
                           .lookup_pair(&machine.locals_sym).unwrap();
    let mut locals_obj = locals_ref.lock();
    let     locals     = &mut locals_obj.meta_mut().members;

    locals.push_pair(machine.symbol("specification"), specification);
  }

  /// Start running the Suite with all of the known rules up to this point.
  ///
  /// Stops the Machine once the Suite has completed.
  pub fn run(&self, reactor: &mut Reactor) {
    let rules = self.rules.lock();

    println!("1..{}", rules.deref().len());

    for (index, rule) in rules.deref().iter().enumerate() {
      rule.start(self, reactor, index);
    }

    let suite = self.clone();
    reactor.on_stall(proc(reactor) {
      for rule in suite.rules.lock().iter() {
        if rule.result.is_none() {
          match rule.eventually.clone() {
            Some(eventually) =>
              reactor.stage(eventually.clone(), eventually),
            None => ()
          }
        }
      }

      reactor.on_stall(proc(reactor) {
        reactor.stop();
      });
    });
  }

  fn rule_alien(&self) -> ObjectRef {
    let data = box RuleAlienData {
      suite:          self.clone(),

      caller:         None,
      name:           None,

      rule:           None,

      got_eventually: false,
      completed:      false
    };

    Alien::create("rule", rule_routine, data)
  }
}

#[deriving(Clone, PartialEq, Eq, Show)]
struct Rule {
  name:       String,
  body:       ObjectRef,
  eventually: Option<ObjectRef>,
  result:     Option<RuleResult>
}

impl Rule {
  fn start(&self, suite: &Suite, reactor: &mut Reactor, index: uint) {
    let pass =
      Alien::create("pass",
                    set_rule_result_routine,
                    box SetRuleResultAlienData {
                      suite: suite.clone(),
                      rule:  index,
                      to:    Pass
                    });

    let fail =
      Alien::create("fail",
                    set_rule_result_routine,
                    box SetRuleResultAlienData {
                      suite: suite.clone(),
                      rule:  index,
                      to:    Fail
                    });

    // Add pass and fail to locals of `body`
    {
      let machine = reactor.machine();

      let body_locals =
        self.body.lock().meta_mut().members
          .lookup_pair(&machine.locals_sym)
          .expect("Execution is missing locals!");

      let mut body_locals = body_locals.lock();

      body_locals.meta_mut().members
        .push_pair(machine.symbol("pass"), pass.clone());
      body_locals.meta_mut().members
        .push_pair(machine.symbol("fail"), fail.clone());
    }

    // Stage `body`
    reactor.stage(self.body.clone(), self.body.clone());

    // Handle `eventually`
    match self.eventually {
      Some(ref eventually) => {
        // Add pass and fail to locals of `eventually`
        {
          let machine = reactor.machine();

          let eventually_locals =
            eventually.lock().meta_mut().members
              .lookup_pair(&machine.locals_sym)
              .expect("Execution is missing locals!");

          let mut eventually_locals = eventually_locals.lock();

          eventually_locals.meta_mut().members
            .push_pair(machine.symbol("pass"), pass.clone());
          eventually_locals.meta_mut().members
            .push_pair(machine.symbol("fail"), fail.clone());
        }
      },
      _ => ()
    }
  }

  fn set_result(&mut self, index: uint, result: RuleResult) {
    self.result = Some(result);

    match result {
      Pass =>
        println!("ok {} - {:s}", index + 1, self.name),
      Fail =>
        println!("not ok {} - {:s}", index + 1, self.name)
    }
  }
}

#[deriving(Clone, PartialEq, Eq, Show)]
enum RuleResult {
  Pass,
  Fail
}

#[deriving(Clone)]
struct RuleAlienData {
  suite:          Suite,

  caller:         Option<ObjectRef>,
  name:           Option<ObjectRef>,

  rule:           Option<uint>,

  got_eventually: bool,
  completed:      bool
}

fn rule_routine<'a>(
                mut alien: TypedRefGuard<'a, Alien>,
                reactor:   &mut Reactor,
                response:  ObjectRef) {

  let caller;
  {
    let data = alien.data.downcast_mut::<RuleAlienData>().unwrap();

    let add_caller_locals_to = |caller: &ObjectRef, dest: &ObjectRef| {
      let caller_locals_members = {
        let caller_locals =
          caller.lock().meta().members
            .lookup_pair(&reactor.machine().locals_sym)
            .expect("Execution is missing locals!");

        caller_locals.lock().meta().members.clone()
      };

      let dest_locals =
        dest.lock().meta().members
          .lookup_pair(&reactor.machine().locals_sym)
          .expect("Execution is missing locals!"); // FIXME: omfg, DRY this

      let mut dest_locals = dest_locals.lock();

      dest_locals.meta_mut().members = caller_locals_members;
    };

    if data.completed {
      // Do nothing
      return

    } else if data.caller.is_none() {
      data.caller = Some(response);

    } else if data.name.is_none() {
      if response.symbol_ref().is_none() {
        warn!("expected name: {} to be a Symbol", response);
        return
      }

      data.name = Some(response);

    } else if data.rule.is_none() {
      let mut rules = data.suite.rules.lock();

      data.rule = Some(rules.len());

      let body = response;

      add_caller_locals_to(data.caller.get_ref(), &body);

      rules.push(Rule {
        name:       (**data.name.get_ref().symbol_ref().unwrap()).clone(),
        body:       body,
        eventually: None,
        result:     None
      });

    } else if !data.got_eventually {
      match response.symbol_ref() {
        Some(sym) if sym.as_slice() == "eventually" => {
          data.got_eventually = true;
        },

        _ => {
          warn!("expected 'eventually'");
          return
        }
      }
    } else {
      let mut rules = data.suite.rules.lock();

      let eventually = response;

      add_caller_locals_to(data.caller.get_ref(), &eventually);

      rules.get_mut(data.rule.unwrap()).eventually = Some(eventually);
    }

    caller = data.caller.get_ref().clone();
  }

  reactor.stage(caller, alien.unlock().clone())
}

#[deriving(Clone)]
struct SetRuleResultAlienData {
  suite: Suite,
  rule:  uint,
  to:    RuleResult
}

fn set_rule_result_routine<'a>(
                            mut alien: TypedRefGuard<'a, Alien>,
                            _reactor:  &mut Reactor,
                            _response: ObjectRef) {

  let data = alien.data.downcast_mut::<SetRuleResultAlienData>().unwrap();

  let mut rules = data.suite.rules.lock();

  rules.get_mut(data.rule).set_result(data.rule, data.to.clone());
}
