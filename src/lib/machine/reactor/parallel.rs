use super::Reactor;
use super::realize;

use machine::Machine;

use object::{ObjectRef, Cache};

use std::collections::{Deque, RingBuf};
use std::mem::replace;
use std::vec::unzip;
use std::sync::{Arc, Mutex};
use std::sync::atomics::{AtomicBool, AtomicUint, SeqCst};
use std::task;

enum ReactorMessage {
  Do(proc (&mut ParallelReactor): Send),
  Stage(ObjectRef, ObjectRef),
  Stall,
  Stop
}

/// A pool of `ParallelReactor`s running in parallel.
///
/// The number of reactors must be configured at creation and can not be
/// dynamically configured.
///
/// # Warning
///
/// `ParallelReactor` is in an early stage of development and may not comply
/// with the Nucleus spec completely at any given time. It may also experience
/// random bugs or failures due to its complexity.
#[deriving(Clone)]
pub struct ReactorPool {
  /// The machine context the pool operates within.
  machine:        Machine,

  /// The next channel to use within this instance. Incremented automatically
  /// for round robin distribution.
  next:           uint,

  /// The index on `channels` of the reactor that owns this `ReactorPool`
  /// instance, if any.
  me:             Option<uint>,

  /// Senders to all reactors in the pool, including this one (if owned).
  channels:       Vec<Sender<ReactorMessage>>,

  /// Keeps a count of all reactors that are waiting for messages within the
  /// pool. It being equal to the total number of reactors in the pool is a
  /// condition for stall detection.
  waiting:        Arc<AtomicUint>,

  /// Keeps a count of all messages that have been sent out but not received
  /// yet. It being zero is also a condition for stall detection.
  ///
  /// This is necessary because it's possible for `waiting` to be equal to the
  /// total number of reactors even though one or more of the reactors that are
  /// waiting for a message have a message available but haven't been scheduled
  /// to handle it yet.
  pending:        Arc<AtomicUint>,

  /// Decides whether to notify other reactors if a stall is detected. Set to
  /// true every time the `stagings` queue within any instance is found to be
  /// non-empty. Set to false once a `Stall` message is sent out.
  notify_stall:   Arc<AtomicBool>,

  /// Determines how many reactors have yet to exit. The condition variable is
  /// used to wait/signal.
  stop_sig:       Arc<Mutex<uint>>
}

impl ReactorPool {
  /// Creates a new `ReactorPool` and spawns the given number of
  /// `ParallelReactor`s.
  ///
  /// The number of reactors can not be changed after the pool is spawned.
  pub fn spawn(machine: Machine, reactors: uint) -> ReactorPool {
    if reactors < 2 {
      fail!("must spawn at least two ParallelReactors!");
    }

    let (senders, receivers) =
      unzip(range(0, reactors).map(|_| channel::<ReactorMessage>()));

    let pool = ReactorPool {
      machine:  machine,

      next:         0,
      me:           None,
      channels:     senders,

      waiting:      Arc::new(AtomicUint::new(0)),
      pending:      Arc::new(AtomicUint::new(0)),
      notify_stall: Arc::new(AtomicBool::new(true)),

      stop_sig:     Arc::new(Mutex::new(reactors))
    };

    for (index, receiver) in receivers.move_iter().enumerate() {
      let mut pool = pool.clone();

      pool.me = Some(index);

      ParallelReactor::spawn(receiver, pool)
    }

    pool
  }

  /// Wait for all reactors to stop.
  pub fn wait(&self) {
    let stop_sig = self.stop_sig.lock();

    while *stop_sig > 0 {
      stop_sig.cond.wait();
    }
  }

  /// Tell all reactors to stop.
  pub fn stop(&self) {
    self.pending.fetch_add(self.len(), SeqCst);

    for channel in self.channels.iter() {
      let _ = channel.send_opt(Stop);
    }
  }

  /// Run a procedure on one of the reactors in this pool.
  ///
  /// Which reactor is chosen is not defined; it could be any of them.
  pub fn on_reactor(&mut self, block: proc (&mut ParallelReactor): Send) {
    self.pending.fetch_add(1, SeqCst);

    let _ = self.next_channel().send_opt(Do(block));
  }

  /// Get the next channel in round robin order.
  ///
  /// If owned, skips the reactor that owns this `ReactorPool` instance.
  fn next_channel(&mut self) -> &Sender<ReactorMessage> {
    let next;

    if Some(self.next) == self.me {
      next = (self.next + 1) % self.len();

      self.next = (self.next + 2) % self.len();
    } else {
      next = self.next;

      self.next = (self.next + 1) % self.len();
    }

    &self.channels[next]
  }
}

impl Collection for ReactorPool {
  fn len(&self) -> uint {
    self.channels.len()
  }
}

/// A reactor that runs in parallel with other reactors.
///
/// This reactor can't be created directly. Use `ReactorPool` to initialize
/// several at once.
///
/// # Warning
///
/// This reactor is in an early stage of development and may not comply with the
/// Nucleus spec completely at any given time. It may also experience random
/// bugs or failures due to its complexity.
pub struct ParallelReactor {
  /// Messages for the reactor.
  receiver:       Receiver<ReactorMessage>,

  /// The pool the reactor belongs to.
  pool:           ReactorPool,

  /// A queue of `stage()` calls assigned to the reactor.
  stagings:       RingBuf<(ObjectRef, ObjectRef)>,

  /// Procedures to be called in the event the pool encounters a stall.
  stall_handlers: Vec<proc (&mut Reactor)>,

  /// Our local cache.
  cache:          Cache
}

impl ParallelReactor {
  fn spawn(receiver: Receiver<ReactorMessage>, pool: ReactorPool) {
    task::spawn(proc () {
      let mut reactor = ParallelReactor {
        receiver:       receiver,
        pool:           pool,
        stagings:       RingBuf::new(),
        stall_handlers: Vec::new(),
        cache:          Cache::new_parallel()
      };

      reactor.run()
    })
  }

  fn run(&mut self) {
    debug!("ParallelReactor started");

    'stop: loop {
      // Process all of the messages available to us immediately, but don't
      // wait.
      'receive: loop {
        match self.receiver.try_recv() {
          Ok(message) => {
            self.pool.pending.fetch_sub(1, SeqCst);

            if !self.handle_message(message) { break 'stop }
          },

          Err(_) =>
            break 'receive
        }
      }

      // If we have work to do, do it. Otherwise check to see if all reactors
      // are stalled, and if so try to notify; if not, wait for a message.
      if !self.stagings.is_empty() {
        // Since we have work, set notify_stall to true so that stall
        // notifications will happen if we find ourselves without work.
        self.pool.notify_stall.store(true, SeqCst);

        // Iterate through the work, being careful to only take how many there
        // were initially.
        let stagings_len = self.stagings.len();

        for _ in range(0, stagings_len) {
          let (execution, response) = self.stagings.pop_front().unwrap();

          realize(self, execution, response)
        }
      } else {
        let waiting = self.pool.waiting.fetch_add(1, SeqCst) + 1;
        let pending = self.pool.pending.load(SeqCst);

        debug!("waiting: {}/{}, pending: {}",
               waiting, self.pool.len(), pending);

        // Only attempt to notify the reactors if they are all waiting *and* all
        // of the channels are empty (represented by `pending == 0`).
        //
        // If both of these are true, then nothing else could possibly change,
        // so this is a safe assumption.
        if waiting == self.pool.len() && pending == 0 {

          // Only notify if no one else has notified yet.
          if self.pool.notify_stall.swap(false, SeqCst) {
            self.pool.pending.fetch_add(self.pool.len(), SeqCst);

            for channel in self.pool.channels.iter() {
              let _ = channel.send_opt(Stall);
            }
          }
        }

        let message = self.receiver.recv();

        self.pool.pending.fetch_sub(1, SeqCst);
        self.pool.waiting.fetch_sub(1, SeqCst);

        if !self.handle_message(message) { break 'stop }
      }
    }

    debug!("ParallelReactor stopped");

    let mut stop_sig = self.pool.stop_sig.lock();
    
    *stop_sig -= 1;
    
    stop_sig.cond.broadcast();
  }

  fn handle_message(&mut self, message: ReactorMessage) -> bool {
    match message {
      Do(block) =>
        block(self),

      Stage(execution, response) =>
        self.stagings.push((execution, response)),

      Stall =>
        self.stall(),

      Stop =>
        return false
    }

    true
  }

  fn stall(&mut self) {
    let stall_handlers = replace(&mut self.stall_handlers, Vec::new());

    for handler in stall_handlers.move_iter() {
      handler(&mut *self)
    }
  }
}

impl Reactor for ParallelReactor {
  fn stage(&mut self, execution: ObjectRef, response: ObjectRef) {
    if self.stagings.is_empty() {
      self.stagings.push((execution, response));
    } else {
      self.pool.pending.fetch_add(1, SeqCst);

      // We don't really care whether this succeeds or not -- if it doesn't, the
      // reactors are stopping so it wouldn't matter.
      let _ = self.pool.next_channel().send_opt(Stage(execution, response));
    }
  }

  fn on_stall(&mut self, handler: proc (&mut Reactor)) {
    self.stall_handlers.push(handler)
  }

  fn stop(&mut self) {
    self.pool.stop()
  }

  fn machine(&self) -> &Machine {
    &self.pool.machine
  }

  fn cache(&mut self) -> &mut Cache {
    &mut self.cache
  }
}
