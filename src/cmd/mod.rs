mod kind;

pub use kind::CmdKind;

use crate::redirect::{Output, Redirect};
use crate::job::Pid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cmd {
    pub kind: CmdKind,
    pub args: Vec<String>,
    pub redirects: Vec<Redirect>,
}

impl Cmd {
    pub fn empty() -> Self {
        Self {
            kind: CmdKind::Empty,
            args: Vec::new(),
            redirects: Vec::new(),
        }
    }

    pub fn exec(self) -> anyhow::Result<Option<Pid>> {
        let Cmd {
            kind,
            args,
            redirects,
        } = self;
        let output = Output::from(redirects)?;
        kind.exec(args, output)
    }
}

