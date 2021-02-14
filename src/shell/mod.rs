mod parse;
use parse::parse_line;

use crate::job::Pid;
use anyhow::Context;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job(Option<Cmd>);

impl Job {
    pub fn parse<T: AsRef<str>>(input: T) -> anyhow::Result<Self> {
        parse_line(input)
    }

    pub fn exec(&self) -> anyhow::Result<Option<Pid>> {
        match self.0 {
            Some(ref cmd) => cmd.exec(),
            None => Ok(None),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cmd {
    kind: CmdKind,
    args: Vec<String>,
}

impl Cmd {
    pub fn exec(&self) -> anyhow::Result<Option<Pid>> {
        Ok(match self.kind {
            CmdKind::Exit => {
                let code = match self.args.len() {
                    0 => 0,
                    1 => self.args[0].parse::<i32>().context("Failed to parse a number.")?,
                    _ => anyhow::bail!("Unnexpected args number."),
                };
                std::process::exit(code);
            }
            CmdKind::Cd => {
                let path = match self.args.len() {
                    0 => std::env::var("HOME").context("Failed to get the home directory.")?,
                    1 => self.args[0].clone(),
                    _ => anyhow::bail!("Unexpected args number."),
                };

                std::env::set_current_dir(path).context("Failed to set current dir.")?;

                None
            }
            CmdKind::Fg => {
                if self.args.len() != 1 {
                    anyhow::bail!("Unexpected args number.");
                }

                let ref id = self.args[0];

                let id = id
                    .parse::<Pid>()
                    .context("Failed to parse a number.")?;
                id.restart()?;

                Some(id)
            }
            CmdKind::Cmd(ref name) => {
                let child = Command::new(name)
                    .args(&self.args)
                    .spawn()
                    .context(format!("Invalid command: {}", name))?;
                Some((child.id() as i32).into())
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CmdKind {
    Exit,
    Cd,
    Fg,
    Cmd(String),
}

impl CmdKind {
    pub fn new<T: AsRef<str>>(name: T) -> Self {
        match name.as_ref() {
            "exit" => Self::Exit,
            "cd" => Self::Cd,
            "fg" => Self::Fg,
            s => Self::Cmd(s.into()),
        }
    }
}
