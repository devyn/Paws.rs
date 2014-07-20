use util::work_queue::*;

use std::sync::Arc;
use std::task::TaskBuilder;
use native::NativeTaskBuilder;

#[test]
fn consume_from_same_task() {
  let queue = WorkQueue::<&'static str>::new(1);

  let strings = ["a", "b", "c"];

  for string in strings.iter() {
    queue.push(*string);
  }

  for string in strings.iter() {
    match queue.shift() {
      Work(string2) => assert!(string2 == *string),
      Stalled       => fail!("unexpected Stalled, expecting Work"),
      Ended         => fail!("unexpected Ended, expecting Work")
    }
  }
}

#[test]
fn consume_from_other_task() {
  let queue  = WorkQueue::<&'static str>::new(1);
  let queue2 = queue.clone();

  let strings  = Arc::new(["a", "b", "c"]);
  let strings2 = strings.clone();

  TaskBuilder::new().native().spawn(proc() {
    for string in strings2.iter() {
      queue2.push(*string);
    }
  });

  for string in strings.iter() {
    'stall: loop {
      match queue.shift() {
        Work(string2) => {
          assert!(string2 == *string,
                  "\"{}\" != \"{}\"", string2, *string);
          break 'stall
        }

        Stalled =>
          // We don't care about stalls here; stalls will naturally occur when
          // racing with a producer, which should never matter when dealing with
          // this queue. (If it does, it will be a simple fix.)
          continue 'stall,

        Ended =>
          fail!("unexpected Ended, expecting Work")
      }
    }
  }
}

#[test]
fn end_queue_broadcast() {
  let queue   = WorkQueue::<uint>::new(2);
  let queue2  = queue.clone();
  let queue3  = queue.clone();

  let task2 = TaskBuilder::new().native().try_future(proc() {
    loop {
      match queue2.shift() {
        Ended => break,
        _     => ()
      }
    }
  });

  let task3 = TaskBuilder::new().native().try_future(proc() {
    loop {
      match queue3.shift() {
        Ended => break,
        _     => ()
      }
    }
  });

  for n in range(0u, 100) {
    queue.push(n);
  }
  queue.end();

  task2.unwrap().ok().unwrap();
  task3.unwrap().ok().unwrap();
}

#[test]
fn single_worker_stall() {
  let queue = WorkQueue::<uint>::new(1);

  for n in range(0u, 100) {
    queue.push(n);
  }

  let mut max_n   = 0u;

  loop {
    match queue.shift() {
      Work(n) => max_n = n,
      Stalled => break,
      Ended   => fail!("unexpected Ended that shouldn't be possible.")
    }
  }

  assert!(max_n == 99, "expected max_n to be 99, got {}", max_n);
}

#[test]
fn multi_worker_stall() {
  let queue   = WorkQueue::<uint>::new(2);
  let queue2  = queue.clone();
  let queue3  = queue.clone();

  for n in range(0u, 100) {
    queue.push(n);
  }

  let task2 = TaskBuilder::new().native().try_future(proc() {
    loop {
      match queue2.shift() {
        Work(_) => (),
        Stalled => queue2.end(),
        Ended   => break
      }
    }
  });

  let task3 = TaskBuilder::new().native().try_future(proc() {
    loop {
      match queue3.shift() {
        Work(_) => (),
        Stalled => queue3.end(),
        Ended   => break
      }
    }
  });

  task2.unwrap().ok().unwrap();
  task3.unwrap().ok().unwrap();
}
