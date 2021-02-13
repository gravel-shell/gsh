extern crate rustyline;

mod sighook;

use std::process::Command;

use anyhow::Context;
use rustyline::{error::ReadlineError, Editor};

use crate::jobs::{CurPid, Pid};

fn fg(args: Vec<&str>) -> anyhow::Result<Pid> {
    if args.len() != 1 {
        anyhow::bail!("Unexpected args number.");
    }

    let id = args[0]
        .parse::<Pid>()
        .context(format!("Invalid process id: {}", args[0]))?;
    id.restart()?;

    Ok(id)
}

fn cmd(name: &str, args: Vec<&str>) -> anyhow::Result<Pid> {
    let child = Command::new(name)
        .args(args)
        .spawn()
        .context(format!("Invalid command: {}", name))?;
    Ok((child.id() as i32).into())
}

pub fn prompt() -> anyhow::Result<()> {
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
                    continue;
                }
            },
            Some(name) => match cmd(name, line.collect()) {
                Ok(id) => id,
                Err(e) => {
                    eprintln!("{}", e);
                    continue;
                }
            },
            None => continue,
        };

        child_id.store(id)?;

        eprintln!("{}", id.wait()?);

        child_id.reset()?;
    }
    Ok(())
}
