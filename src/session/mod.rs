use std::process::Command;
use anyhow::Context;
use crate::job::{Pid, CurPid};

pub struct Session<T> {
    reader: T,
    cur_pid: CurPid,
}

pub trait Reader: Sized {
    fn init(cur_pid: &CurPid) -> anyhow::Result<Self>;
    fn next_line(&mut self) -> anyhow::Result<String>;
}

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

impl<T: Reader> Session<T> {
    pub fn new() -> anyhow::Result<Self> {
        let cur_pid = CurPid::new();
        Ok(Self {
            reader: T::init(&cur_pid)?,
            cur_pid,
        })
    }

    pub fn next(&mut self) -> anyhow::Result<bool> {
        let line = match self.reader.next_line() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Readline Error: {}", e);
                return Ok(true);
            }
        };

        let mut line = line.split_whitespace();

        let id = match line.next() {
            Some("exit") => return Ok(false),
            Some("fg") => match fg(line.collect()) {
                Ok(id) => id,
                Err(e) => {
                    eprintln!("{}", e);
                    return Ok(true);
                }
            },
            Some(name) => match cmd(name, line.collect()) {
                Ok(id) => id,
                Err(e) => {
                    eprintln!("{}", e);
                    return Ok(true);
                }
            },
            None => return Ok(true),
        };

        self.cur_pid.store(id)?;

        eprintln!("{}", id.wait()?);

        self.cur_pid.reset()?;

        Ok(true)
    }

    pub fn all(&mut self) -> anyhow::Result<()> {
        loop {
            if self.next()? {
                continue
            } else {
                break
            }
        }

        Ok(())
    }
}
