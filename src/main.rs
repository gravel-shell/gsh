use std::convert::TryFrom;
use std::fmt;
use std::process::Command;
use std::sync::{Arc, Mutex};

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

fn safe_waitid(pid: Pid, flag: WaitPidFlag) -> nix::Result<Status> {
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
        return Err(nix::Error::Sys(nix::errno::Errno::last()));
    }

    Ok(match code {
        libc::CLD_EXITED => Status::Exited(status),
        _ => Status::Signaled(Signal::try_from(status).expect("Unnexpected signal.")),
    })
}

fn main() {
    let mut signals =
        Signals::new(&[signal::SIGINT, signal::SIGTSTP]).expect("Failed to set signals.");
    let child_id: Arc<Mutex<Option<i32>>> = Arc::new(Mutex::new(None));

    let child_signal = Arc::clone(&child_id);
    thread::spawn(move || {
        for sig in signals.forever() {
            let child = child_signal.lock().expect("Failed to get the child id.");
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
            Some("fg") => {
                let id = match line.next() {
                    Some(s) => s,
                    None => {
                        eprintln!("\"fg\" requires an process id.");
                        continue;
                    }
                };
                let id = match id.parse::<i32>() {
                    Ok(i) => i,
                    Err(_) => {
                        eprintln!("Invalid process id: {}", id);
                        continue;
                    }
                };
                match kill(Pid::from_raw(id), Signal::SIGCONT) {
                    Ok(_) => {}
                    Err(_) => {
                        eprintln!("Failed to continue the process: {}", id);
                        continue;
                    }
                };
                id
            }
            Some(name) => match Command::new(name).args(line).spawn() {
                Ok(child) => child,
                Err(_) => {
                    eprintln!("Invalid command: {}", name);
                    continue;
                }
            }
            .id() as i32,
            None => continue,
        };

        *Arc::clone(&child_id).lock().expect("Failed to get child.") = Some(id);

        eprintln!(
            "{}",
            safe_waitid(
                Pid::from_raw(id),
                WaitPidFlag::WEXITED | WaitPidFlag::WSTOPPED,
            )
            .expect("Failed to wait the process")
        );

        *Arc::clone(&child_id).lock().expect("Failed to get child.") = None;
    }
}
