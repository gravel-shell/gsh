extern crate rustyline;
extern crate signal_hook;

use super::Reader;
use crate::job::SharedJobs;
use anyhow::Context;
use rustyline::{error::ReadlineError, Editor};
use signal_hook::consts::signal;
use signal_hook::iterator::Signals;
use std::thread;

#[derive(Debug)]
pub struct PromptReader(Editor<()>);

impl Reader for PromptReader {
    fn init(&mut self, jobs: &SharedJobs) -> anyhow::Result<()> {
        sighook(jobs)
    }

    fn next_line(&mut self) -> anyhow::Result<Option<String>> {
        match self.0.readline("$ ") {
            Ok(s) => Ok(Some(s)),
            Err(ReadlineError::Interrupted) => Ok(Some(String::new())),
            Err(ReadlineError::Eof) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn more_line(&mut self) -> anyhow::Result<Option<String>> {
        match self.0.readline("... ") {
            Ok(s) => Ok(Some(s)),
            Err(ReadlineError::Eof) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}

impl PromptReader {
    pub fn new() -> Self {
        Self(Editor::new())
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
