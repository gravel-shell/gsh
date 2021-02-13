extern crate anyhow;
extern crate nix;
extern crate rustyline;

mod raw;
mod sighook;

use std::process::Command;
use std::sync::{Arc, Mutex};

use anyhow::Context;
use nix::sys::signal::{kill, Signal};
use nix::sys::wait::WaitPidFlag;
use nix::unistd::Pid;
use rustyline::{error::ReadlineError, Editor};

fn fg(args: Vec<&str>) -> anyhow::Result<i32> {
    if args.len() != 1 {
        anyhow::bail!("Unexpected args number.");
    }

    let id = args[0].parse::<i32>().context(format!("Invalid process id: {}", args[0]))?;
    kill(Pid::from_raw(id), Signal::SIGCONT).context(format!("Failed to restart the process: {}", id))?;

    Ok(id)
}

fn cmd(name: &str, args: Vec<&str>) -> anyhow::Result<i32> {
    let child = Command::new(name).args(args).spawn().context(format!("Invalid command: {}", name))?;
    Ok(child.id() as i32)
}

fn inner_main() -> anyhow::Result<()> {
    let child_id: Arc<Mutex<Option<i32>>> = Arc::new(Mutex::new(None));
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

        *Arc::clone(&child_id).lock().expect("Failed to get child.") = Some(id);

        eprintln!(
            "{}",
            raw::waitid(
                Pid::from_raw(id),
                WaitPidFlag::WEXITED | WaitPidFlag::WSTOPPED
            )?
        );

        *Arc::clone(&child_id).lock().expect("Failed to get child.") = None;
    }
    Ok(())
}

fn main() {
    inner_main().unwrap_or_else(|e| {
        eprintln!("{}", e);
    })
}
