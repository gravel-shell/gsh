mod parse;
use parse::parse_line;

use crate::job::Pid;
use anyhow::Context;
use std::process::{Command, Stdio};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
    cmd: Option<Cmd>,
}

impl Job {
    pub fn parse<T: AsRef<str>>(input: T) -> anyhow::Result<Self> {
        parse_line(input)
    }

    pub fn exec(self) -> anyhow::Result<Option<Pid>> {
        match self.cmd {
            Some(cmd) => cmd.exec(),
            None => Ok(None),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cmd {
    kind: CmdKind,
    args: Vec<String>,
    redirects: Vec<Redirect>,
}

impl Cmd {
    pub fn exec(self) -> anyhow::Result<Option<Pid>> {
        let Cmd {
            kind,
            args,
            redirects,
        } = self;
        let output = Output::from(redirects);
        Ok(match kind {
            CmdKind::Exit => {
                let code = match args.len() {
                    0 => 0,
                    1 => args[0]
                        .parse::<i32>()
                        .context("Failed to parse a number.")?,
                    _ => anyhow::bail!("Unnexpected args number."),
                };
                std::process::exit(code);
            }
            CmdKind::Cd => {
                let path = match args.len() {
                    0 => std::env::var("HOME").context("Failed to get the home directory.")?,
                    1 => args.into_iter().next().unwrap(),
                    _ => anyhow::bail!("Unexpected args number."),
                };

                std::env::set_current_dir(path).context("Failed to set current dir.")?;

                None
            }
            CmdKind::Fg => {
                if args.len() != 1 {
                    anyhow::bail!("Unexpected args number.");
                }

                let id = args.into_iter().next().unwrap();

                let id = id.parse::<Pid>().context("Failed to parse a number.")?;
                id.restart()?;

                Some(id)
            }
            CmdKind::Cmd(ref name) => {
                let mut child = Command::new(name);
                child.args(args);

                if output.stdin.is_some() {
                    child.stdin(Stdio::piped());
                }

                if output.stdout.is_some() {
                    child.stdout(Stdio::piped());
                }

                let child = child
                    .spawn()
                    .context(format!("Invalid command: {}", name))?;

                let id = Pid::from(child.id() as i32);

                if let Some(s) = output.stdin {
                    std::io::copy(
                        &mut std::fs::File::open(s).context("Failed to open the file")?,
                        &mut child.stdin.unwrap(),
                    )
                    .context("Failed to redirect")?;
                }

                if let Some(s) = output.stdout {
                    std::io::copy(
                        &mut child.stdout.unwrap(),
                        &mut std::fs::File::create(s).context("Failed to open the file")?,
                    )
                    .context("Failed to redirect")?;
                }

                Some(id)
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Redirect {
    Stdin(String),
    Stdout(String),
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Output {
    stdin: Option<String>,
    stdout: Option<String>,
}

impl Output {
    fn from(reds: Vec<Redirect>) -> Self {
        let mut res = Self::default();
        reds.into_iter().fold(&mut res, |acc, red| {
            match red {
                Redirect::Stdin(s) => acc.stdin = Some(s),
                Redirect::Stdout(s) => acc.stdout = Some(s),
            }
            acc
        });
        res
    }
}
