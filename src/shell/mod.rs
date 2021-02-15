mod builtin;
mod parse;
mod redirect;

pub use redirect::{Output, RedIn, RedOut, Redirect};

use parse::parse_line;

use crate::job::Pid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cmd {
    kind: CmdKind,
    args: Vec<String>,
    redirects: Vec<Redirect>,
}

impl Cmd {
    pub fn empty() -> Self {
        Self {
            kind: CmdKind::Empty,
            args: Vec::new(),
            redirects: Vec::new(),
        }
    }

    pub fn parse<T: AsRef<str>>(input: T) -> anyhow::Result<Self> {
        parse_line(input)
    }

    pub fn exec(self) -> anyhow::Result<Option<Pid>> {
        let Cmd {
            kind,
            args,
            redirects,
        } = self;
        let output = Output::from(redirects)?;
        Ok(match kind {
            CmdKind::Empty => None,
            CmdKind::Exit => {
                builtin::exit(args)?;
                None
            }
            CmdKind::Cd => {
                builtin::cd(args)?;
                None
            }
            CmdKind::Fg => Some(builtin::fg(args)?),
            CmdKind::Cmd(ref name) => Some(builtin::cmd(name, args, output)?),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CmdKind {
    Empty,
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
