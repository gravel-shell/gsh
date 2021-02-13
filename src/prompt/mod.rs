mod sighook;
mod readline;

use std::process::Command;

use anyhow::Context;

use crate::jobs::{CurPid, Pid};
use readline::Reader;

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

    let mut readline = Reader::new();

    loop {
        let line = match readline.read() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Readline Error: {}", e);
                continue
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
