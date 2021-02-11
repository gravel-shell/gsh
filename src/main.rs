use std::process::Command;
use std::io::{stdin, stderr, Write};

fn main() {
    loop {
        eprint!("$ ");
        stderr().flush().expect("Failed to flush stderr.");

        let mut s = String::new();
        stdin().read_line(&mut s).expect("Failed to read stdin");

        let mut s = s.split_whitespace();
        let mut child = match s.next() {
            Some("exit") => break,
            Some(name) => Command::new(name).args(s).spawn().expect("Failed to spawn a process."),
            None => continue,
        };

        child.wait().expect("Command wasn't running.");
    }
}
