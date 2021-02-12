use std::process::Command;

use rustyline::{Editor, error::ReadlineError};

fn main() {
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

        child.wait().expect("Command wasn't running.");
    }
}
