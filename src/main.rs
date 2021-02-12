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

fn safe_waitid(pid: Pid, flag: WaitPidFlag) -> Status {
    let (code, status) = unsafe {
        let mut siginfo = std::mem::zeroed();
        waitid(P_PID, pid.as_raw() as u32, &mut siginfo, flag.bits());
        let siginfo = siginfo as siginfo_t;
        (siginfo.si_code as i32, siginfo.si_status() as i32)
    };

    match code {
        libc::CLD_EXITED => Status::Exited(status),
        _ => Status::Signaled(Signal::try_from(status).unwrap()),
    }
}

fn main() {
    let mut signals = Signals::new(&[signal::SIGINT, signal::SIGTSTP]).unwrap();
    let child_id: Arc<Mutex<Option<i32>>> = Arc::new(Mutex::new(None));

    let child_signal = Arc::clone(&child_id);
    thread::spawn(move || {
        for sig in signals.forever() {
            let child = child_signal.lock().unwrap();
            if let Some(id) = *child {
                match sig {
                    signal::SIGINT => {
                        println!("\nInterrupt");
                        kill(Pid::from_raw(id), Signal::SIGINT).unwrap();
                    }
                    signal::SIGTSTP => {
                        println!("\nSuspend: {}", id);
                        kill(Pid::from_raw(id), Signal::SIGSTOP).unwrap();
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
            Err(e) => Err(e).expect("Failed to read line."),
        };

        let mut line = line.split_whitespace();

        let id = match line.next() {
            Some("exit") => break,
            Some("fg") => {
                let id = line.next().unwrap().parse::<i32>().unwrap();
                kill(Pid::from_raw(id), Signal::SIGCONT).unwrap();
                id
            }
            Some(name) => Command::new(name)
                .args(line)
                .spawn()
                .expect("Failed to spawn a process.")
                .id() as i32,
            None => continue,
        };

        *Arc::clone(&child_id).lock().unwrap() = Some(id);

        println!("{}", safe_waitid(
            Pid::from_raw(id),
            WaitPidFlag::WEXITED | WaitPidFlag::WSTOPPED,
        ));

        *Arc::clone(&child_id).lock().unwrap() = None;
    }
}
