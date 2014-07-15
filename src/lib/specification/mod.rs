//! Paws Rulebook-compliant specification interface.
//!
//! The output conforms to the
//! [Test Anything Protocol](http://testanything.org/).

use std::any::AnyMutRefExt;
use std::sync::{Arc, Mutex};

use machine::*;
use object::*;
use object::execution::Execution;
use object::alien::Alien;
use object::thing::Thing;

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
  pub fn expose_to(&self, execution: &mut Execution, machine: &Machine) {
    let mut specification = box Thing::new();

    specification.meta_mut().members
      .push_pair(machine.symbol("rule"), self.rule_alien());

    let specification  = ObjectRef::new_with_tag(specification, 
                                               "(specification)");

    let     locals_ref = execution.meta_mut().members
                           .lookup_pair(&machine.locals_sym).unwrap();
    let mut locals_obj = locals_ref.lock();
    let     locals     = &mut locals_obj.meta_mut().members;

    locals.push_pair(machine.symbol("specification"), specification);
  }

  /// Start running the Suite with all of the known rules up to this point.
  ///
  /// Stops the Machine once the Suite has completed.
  pub fn run(&self, machine: &Machine) {
    let rules = self.rules.lock();

    println!("1..{}", rules.deref().len());

    for (index, rule) in rules.deref().iter().enumerate() {
      rule.start(self, machine, index);
    }

    let suite = self.clone();
    machine.on_stall(proc(machine) {
      for rule in suite.rules.lock().iter() {
        if rule.result.is_none() {
          match rule.eventually.clone() {
            Some(eventually) =>
              machine.enqueue(eventually.clone(), eventually),
            None => ()
          }
        }
      }

      machine.on_stall(proc(machine) {
        machine.stop();
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

    ObjectRef::new(box Alien::new(rule_routine, data))
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
  fn start(&self, suite: &Suite, machine: &Machine, index: uint) {
    let pass = ObjectRef::new(box
      Alien::new(set_rule_result_routine,
                 box SetRuleResultAlienData {
                   suite: suite.clone(),
                   rule:  index,
                   to:    Pass
                 }));

    let fail = ObjectRef::new(box
      Alien::new(set_rule_result_routine,
                 box SetRuleResultAlienData {
                   suite: suite.clone(),
                   rule:  index,
                   to:    Fail
                 }));

    // Add pass and fail to locals of `body`
    {
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
    machine.enqueue(self.body.clone(), self.body.clone());

    // Handle `eventually`
    match self.eventually {
      Some(ref eventually) => {
        // Add pass and fail to locals of `eventually`
        {
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
                machine:   &Machine,
                response:  ObjectRef)
                -> Reaction {

  let caller;
  {
    let data = alien.data.as_mut::<RuleAlienData>().unwrap();

    let add_caller_locals_to = |caller: &ObjectRef, dest: &ObjectRef| {
      let caller_locals_members = {
        let caller_locals =
          caller.lock().meta().members
            .lookup_pair(&machine.locals_sym)
            .expect("Execution is missing locals!");

        caller_locals.lock().meta().members.clone()
      };

      let dest_locals =
        dest.lock().meta().members
          .lookup_pair(&machine.locals_sym)
          .expect("Execution is missing locals!"); // FIXME: omfg, DRY this

      let mut dest_locals = dest_locals.lock();

      dest_locals.meta_mut().members = caller_locals_members;
    };

    if data.completed {
      return Yield

    } else if data.caller.is_none() {
      data.caller = Some(response);

    } else if data.name.is_none() {
      if response.symbol_ref().is_none() {
        warn!("expected name: {} to be a Symbol", response);
        return Yield
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
          return Yield
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

  React(caller, alien.unlock().clone())
}

#[deriving(Clone)]
struct SetRuleResultAlienData {
  suite: Suite,
  rule:  uint,
  to:    RuleResult
}

fn set_rule_result_routine<'a>(
                            mut alien: TypedRefGuard<'a, Alien>,
                            _machine:  &Machine,
                            _response: ObjectRef)
                            -> Reaction {

  let data = alien.data.as_mut::<SetRuleResultAlienData>().unwrap();

  let mut rules = data.suite.rules.lock();

  rules.get_mut(data.rule).set_result(data.rule, data.to.clone());

  Yield
}
