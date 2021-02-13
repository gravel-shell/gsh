use anyhow::Context;
use std::process::Command;
use crate::job::Pid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cmd {
    Fg,
    Cmd(String)
}

impl Cmd {
    pub fn new<T: AsRef<str>>(name: T) -> Self {
        match name.as_ref() {
            "fg" => Self::Fg,
            s => Self::Cmd(s.into()),
        }
    }

    pub fn exec(&self, args: Vec<&str>) -> anyhow::Result<Option<Pid>> {
        Ok(match self {
            Self::Fg => {
                if args.len() != 1 {
                    anyhow::bail!("Unexpected args number.");
                }

                let id = args[0]
                    .parse::<Pid>()
                    .context(format!("Invalid process id: {}", args[0]))?;
                    id.restart()?;

                Some(id)
            }
            Self::Cmd(name) => {
                let child = Command::new(name)
                    .args(args)
                    .spawn()
                    .context(format!("Invalid command: {}", name))?;
                Some((child.id() as i32).into())
            }
        })
    }
}
