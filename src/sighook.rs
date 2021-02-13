extern crate signal_hook;

use std::sync::{Arc, Mutex};
use std::thread;

use anyhow::Context;
use signal_hook::iterator::Signals;
use signal_hook::consts::signal;
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;

pub fn sighook(child_id: &Arc<Mutex<Option<i32>>>) -> anyhow::Result<()> {
    let mut signals = Signals::new(&[signal::SIGINT, signal::SIGTSTP])
        .context("Failed to initialize signals.")?;

    let child_id = Arc::clone(&child_id);
    thread::spawn(move || {
        for sig in signals.forever() {
            let child = child_id.lock().expect("Failed to get the child id.");
            if let Some(id) = *child {
                match sig {
                    signal::SIGINT => {
                        println!("\nInterrupt");
                        kill(Pid::from_raw(id), Signal::SIGINT)
                            .expect("Failed to kill the process.");
                    }
                    signal::SIGTSTP => {
                        println!("\nSuspend: {}", id);
                        kill(Pid::from_raw(id), Signal::SIGSTOP)
                            .expect("Failed to stop the process.");
                    }
                    _ => unreachable!(),
                }
            }
        }
    });
    Ok(())
}

