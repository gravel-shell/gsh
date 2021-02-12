use std::convert::TryFrom;
use std::fmt;
use std::process::Command;
use std::sync::{Arc, Mutex};

use anyhow::Context;
use nix::libc::{self, siginfo_t, waitid, P_PID};
use nix::sys::signal::{kill, Signal};
use nix::sys::wait::WaitPidFlag;
use nix::unistd::Pid;
use rustyline::{error::ReadlineError, Editor};
use signal_hook::{consts::signal, iterator::Signals};
use std::thread;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum Status {
    Exited(i32),
    Signaled(Signal),
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Status::Exited(c) => write!(f, "exited: {}", c),
            Status::Signaled(s) => write!(f, "signaled: {}", s),
        }
    }
}

fn safe_waitid(pid: Pid, flag: WaitPidFlag) -> anyhow::Result<Status> {
    let (code, status, is_error) = unsafe {
        let mut siginfo = std::mem::zeroed();
        let error = waitid(P_PID, pid.as_raw() as u32, &mut siginfo, flag.bits());
        let siginfo = siginfo as siginfo_t;
        (
            siginfo.si_code as i32,
            siginfo.si_status() as i32,
            error == -1,
        )
    };

    if is_error {
        Err(nix::Error::Sys(nix::errno::Errno::last())).context("Failed to wait the process.")?;
    }

    Ok(match code {
        libc::CLD_EXITED => Status::Exited(status),
        _ => Status::Signaled(Signal::try_from(status).context("Unnexpected signal.")?),
    })
}

fn sighook(child_id: &Arc<Mutex<Option<i32>>>) -> anyhow::Result<()> {
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
    sighook(&child_id)?;

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
            safe_waitid(
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
