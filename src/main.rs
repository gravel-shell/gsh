use std::process::Command;
use std::sync::{Arc, Mutex};

use signal_hook::{iterator::Signals, consts::signal};
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use rustyline::{error::ReadlineError, Editor};
use std::thread;

fn main() {
    let mut signals = Signals::new(&[signal::SIGINT, signal::SIGTSTP]).unwrap();
    let child_id: Arc<Mutex<Option<i32>>> = Arc::new(Mutex::new(None));

    let child_signal = Arc::clone(&child_id);
    thread::spawn(move || {
        for sig in signals.forever() {
            let child = child_signal.lock().unwrap();
            if let Some(id) = *child {
                match sig {
                    signal::SIGINT => println!("\nInterrupt"),
                    signal::SIGTSTP => println!("\nSuspend"),
                    _ => unreachable!(),
                }

                kill(Pid::from_raw(id), Signal::SIGINT).unwrap();
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

        let mut child = match line.next() {
            Some("exit") => break,
            Some(name) => Command::new(name)
                .args(line)
                .spawn()
                .expect("Failed to spawn a process."),
            None => continue,
        };

        *Arc::clone(&child_id).lock().unwrap() = Some(child.id() as i32);

        child.wait().expect("Command wasn't running.");

        *Arc::clone(&child_id).lock().unwrap() = None;
    }
}
