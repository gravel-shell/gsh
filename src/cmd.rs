use crate::job::Pid;
use anyhow::Context;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cmd {
    Cd,
    Fg,
    Cmd(String),
}

impl Cmd {
    pub fn new<T: AsRef<str>>(name: T) -> Self {
        match name.as_ref() {
            "cd" => Self::Cd,
            "fg" => Self::Fg,
            s => Self::Cmd(s.into()),
        }
    }

    pub fn exec<T: AsRef<str>>(&self, args: Vec<T>) -> anyhow::Result<Option<Pid>> {
        Ok(match self {
            Self::Cd => {
                let path = match args.len() {
                    0 => std::env::var("HOME").context("Failed to get the home directory.")?,
                    1 => String::from(args[0].as_ref()),
                    _ => anyhow::bail!("Unexpected args number."),
                };

                std::env::set_current_dir(path).context("Failed to set current dir.")?;

                None
            }
            Self::Fg => {
                if args.len() != 1 {
                    anyhow::bail!("Unexpected args number.");
                }

                let id = args[0].as_ref();

                let id = id
                    .parse::<Pid>()
                    .context(format!("Invalid process id: {}", id))?;
                id.restart()?;

                Some(id)
            }
            Self::Cmd(name) => {
                let child = Command::new(name)
                    .args(args.iter().map(|s| s.as_ref()))
                    .spawn()
                    .context(format!("Invalid command: {}", name))?;
                Some((child.id() as i32).into())
            }
        })
    }
}
