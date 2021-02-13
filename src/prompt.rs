extern crate rustyline;
extern crate signal_hook;

use anyhow::Context;
use signal_hook::consts::signal;
use signal_hook::iterator::Signals;
use rustyline::{error::ReadlineError, Editor};
use std::thread;
use crate::session::Reader;
use crate::job::CurPid;

#[derive(Debug)]
pub struct PromptReader(Editor<()>);

impl Reader for PromptReader {
    fn init(cur_pid: &CurPid) -> anyhow::Result<Self> {
        sighook(cur_pid)?;
        Ok(Self(Editor::<()>::new()))
    }

    fn next_line(&mut self) -> anyhow::Result<String> {
        match self.0.readline("$ ") {
            Ok(s) => Ok(s),
            Err(ReadlineError::Interrupted) => Ok(String::new()),
            Err(ReadlineError::Eof) => Ok(String::from("exit")),
            Err(e) => Err(e)?,
        }
    }
}

fn sighook(child_id: &CurPid) -> anyhow::Result<()> {
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
