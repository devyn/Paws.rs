use util::queue::Queue;

use sync::Arc;
use std::comm;
use native;

#[test]
fn consume_from_same_task() {
  let queue = Arc::new(Queue::<&'static str>::new());

  queue.push("a");
  queue.push("b");
  queue.push("c");

  assert!(queue.shift() == Some("a"));
  assert!(queue.shift() == Some("b"));
  assert!(queue.shift() == Some("c"));
}

#[test]
fn consume_from_other_task() {
  let queue  = Arc::new(Queue::<&'static str>::new());
  let queue2 = queue.clone();

  native::task::spawn(proc() {
    queue2.push("a");
    queue2.push("b");
    queue2.push("c");
  });

  assert!(queue.shift() == Some("a"));
  assert!(queue.shift() == Some("b"));
  assert!(queue.shift() == Some("c"));
}

#[test]
fn end_queue_broadcast() {
  let queue   = Arc::new(Queue::<uint>::new());
  let queue2  = queue.clone();
  let queue3  = queue.clone();

  let (exited_tx2, exited_rx) = comm::channel::<()>();
  let exited_tx3 = exited_tx2.clone();

  native::task::spawn(proc() {
    for _ in queue2.iter() {
    }

    exited_tx2.send(());
  });

  native::task::spawn(proc() {
    for _ in queue3.iter() {
    }

    exited_tx3.send(());
  });

  for n in range(0u, 100) {
    queue.push(n);
  }
  queue.end();

  exited_rx.recv();
  exited_rx.recv();
}
