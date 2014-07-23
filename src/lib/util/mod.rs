///! Utility functions and structures.

use std::io::timer::Timer;

pub mod namespace;
pub mod clone;

/// Spawn the given block and fail if the timeout is reached before it
/// completes.
#[allow(dead_code)] // ??
pub fn timeout(msecs: u64, block: proc(): Send) {
  let (complete_tx, complete_rx) = channel::<()>();

  let mut timer = Timer::new().unwrap();

  spawn(proc() {
    block();
    complete_tx.send(());
  });

  let timeout = timer.oneshot(msecs);

  select!(
    () = timeout.recv()     => fail!("timeout ({}ms) reached!", msecs),
    () = complete_rx.recv() => ()
  )
}
