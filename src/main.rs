extern crate anyhow;
extern crate rustyline;

mod raw;
mod sighook;

use std::process::Command;
use std::sync::{Arc, Mutex};

use anyhow::Context;
use rustyline::{error::ReadlineError, Editor};

use raw::Pid;

#[derive(Debug)]
pub struct CurPid(Arc<Mutex<Option<Pid>>>);

impl CurPid {
    fn new() -> Self {
        Self(Arc::new(Mutex::new(None)))
    }

    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }

    fn get(&self) -> anyhow::Result<Option<Pid>> {
        let lock = match self.0.lock() {
            Ok(l) => l,
            Err(e) => anyhow::bail!("Failed to get the lock: {}", e),
        };

        Ok(*lock)
    }

    fn store(&self, pid: Pid) -> anyhow::Result<()> {
        let cloned = Arc::clone(&self.0);
        let mut lock = match cloned.lock() {
            Ok(l) => l,
            Err(e) => anyhow::bail!("Failed to get the lock: {}", e),
        };

        *lock = Some(pid);
        Ok(())
    }

    fn reset(&self) -> anyhow::Result<()> {
        let cloned = Arc::clone(&self.0);
        let mut lock = match cloned.lock() {
            Ok(l) => l,
            Err(e) => anyhow::bail!("Failed to get the lock: {}", e),
        };

        *lock = None;
        Ok(())
    }
}

fn fg(args: Vec<&str>) -> anyhow::Result<Pid> {
    if args.len() != 1 {
        anyhow::bail!("Unexpected args number.");
    }

    let id = args[0].parse::<Pid>().context(format!("Invalid process id: {}", args[0]))?;
    id.restart()?;

    Ok(id)
}

fn cmd(name: &str, args: Vec<&str>) -> anyhow::Result<Pid> {
    let child = Command::new(name).args(args).spawn().context(format!("Invalid command: {}", name))?;
    Ok((child.id() as i32).into())
}

fn inner_main() -> anyhow::Result<()> {
    let child_id = CurPid::new();
    sighook::sighook(&child_id)?;

    let mut readline = Editor::<()>::new();

    loop {
        let line = match readline.readline("$ ") {
            Ok(s) => s,
            Err(ReadlineError::Interrupted) => continue,
            Err(ReadlineError::Eof) => break,
            Err(e) => {
                eprintln!("Failed to read line: {}", e);
                continue;
            }
        };

        let mut line = line.split_whitespace();

        let id = match line.next() {
            Some("exit") => break,
            Some("fg") => match fg(line.collect()) {
                Ok(id) => id,
                Err(e) => {
                    eprintln!("{}", e);
                    continue
                }
            }
            Some(name) => match cmd(name, line.collect()) {
                Ok(id) => id,
                Err(e) => {
                    eprintln!("{}", e);
                    continue
                }
            }
            None => continue,
        };

        child_id.store(id)?;

        eprintln!(
            "{}",
            id.wait()?
        );

        child_id.reset()?;
    }
    Ok(())
}

fn main() {
    inner_main().unwrap_or_else(|e| {
        eprintln!("{}", e);
    })
}
