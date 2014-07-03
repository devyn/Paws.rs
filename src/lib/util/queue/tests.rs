use util::queue::Queue;

use sync::Arc;
use std::task::TaskBuilder;
use native::NativeTaskBuilder;

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

  TaskBuilder::new().native().spawn(proc() {
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

  let task2 = TaskBuilder::new().native().try_future(proc() {
    for _ in queue2.iter() {
    }
  });

  let task3 = TaskBuilder::new().native().try_future(proc() {
    for _ in queue3.iter() {
    }
  });

  for n in range(0u, 100) {
    queue.push(n);
  }
  queue.end();

  task2.unwrap().ok().expect("task2 failed!");
  task3.unwrap().ok().expect("task3 failed!");
}
