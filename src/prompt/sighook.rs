extern crate signal_hook;

use std::thread;

use anyhow::Context;
use signal_hook::consts::signal;
use signal_hook::iterator::Signals;

use crate::CurPid;

pub fn sighook(child_id: &CurPid) -> anyhow::Result<()> {
    let mut signals = Signals::new(&[signal::SIGINT, signal::SIGTSTP])
        .context("Failed to initialize signals.")?;

    let child_id = child_id.clone();
    thread::spawn(move || {
        for sig in signals.forever() {
            let child = child_id.get().unwrap();
            if let Some(id) = child {
                match sig {
                    signal::SIGINT => {
                        id.interrupt().unwrap();
                        println!("\nInterrupt");
                    }
                    signal::SIGTSTP => {
                        id.suspend().unwrap();
                        println!("\nSuspend: {}", id);
                    }
                    _ => unreachable!(),
                }
            }
        }
    });
    Ok(())
}
