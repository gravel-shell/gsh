extern crate rustyline;
extern crate signal_hook;

use crate::job::SharedJobs;
use crate::session::{MoreLine, Reader};
use anyhow::Context;
use rustyline::{error::ReadlineError, Editor};
use signal_hook::consts::signal;
use signal_hook::iterator::Signals;
use std::thread;

#[derive(Debug)]
pub struct PromptReader(Editor<()>);

impl Reader for PromptReader {
    fn init(jobs: &SharedJobs) -> anyhow::Result<Self> {
        sighook(jobs)?;
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

    fn more_line(&mut self) -> anyhow::Result<MoreLine> {
        match self.0.readline("... ") {
            Ok(s) => Ok(MoreLine::Get(s)),
            Err(ReadlineError::Eof) => Ok(MoreLine::Eof),
            Err(e) => Err(e)?,
        }
    }
}

fn sighook(jobs: &SharedJobs) -> anyhow::Result<()> {
    let mut signals = Signals::new(&[signal::SIGINT, signal::SIGTSTP, signal::SIGCHLD])
        .context("Failed to initialize signals.")?;

    let jobs = jobs.clone();
    thread::spawn(move || {
        for sig in signals.forever() {
            let res = jobs.with(|jobs| match sig {
                signal::SIGINT => jobs.sigint(),
                signal::SIGTSTP => jobs.sigtstp(),
                signal::SIGCHLD => jobs.sigchld(),
                _ => unreachable!(),
            });
            match res {
                Ok(()) => (),
                Err(e) => eprintln!("Signal hook: {}", e),
            }
        }
    });
    Ok(())
}
