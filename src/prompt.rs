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
    let mut signals = Signals::new(&[signal::SIGINT, signal::SIGTSTP])
        .context("Failed to initialize signals.")?;

    let jobs = jobs.clone();
    thread::spawn(move || {
        for sig in signals.forever() {
            let mut tmp_jobs = jobs.get().unwrap();
            match sig {
                signal::SIGINT => {
                    let proc = tmp_jobs.pop(0);
                    if let Some(id) = proc {
                        id.interrupt().unwrap();
                        println!("\nInterrupt");
                    }
                }
                signal::SIGTSTP => {
                    // proc.suspend().unwrap();
                    // cur_proc.store(proc).unwrap();
                    // println!("\nSuspend: {}", proc);
                }
                _ => unreachable!(),
            }
            jobs.store(tmp_jobs).unwrap();
        }
    });
    Ok(())
}
