mod kind;
mod redirect;

pub use kind::CmdKind;
pub use redirect::Redirects;

use crate::job::Jobs;
use crate::parse::{Arg, Command as ParseCmd, Redirect};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    pub kind: CmdKind,
    pub args: Vec<String>,
    pub reds: Vec<Redirect>,
}

impl From<ParseCmd> for Command {
    fn from(cmd: ParseCmd) -> Self {
        let ParseCmd {
            name,
            args: arg_reds,
        } = cmd;
        let kind = CmdKind::new(name);
        let mut args = Vec::new();
        let mut reds = Vec::new();
        for arg in arg_reds {
            match arg {
                Arg::Arg(s) => {
                    args.push(s);
                }
                Arg::Redirect(r) => {
                    reds.push(r);
                }
            }
        }

        Self { kind, args, reds }
    }
}

impl Command {
    pub fn exec(self, jobs: &mut Jobs) -> anyhow::Result<()> {
        let Command { kind, args, reds } = self;
        let reds = Redirects::new(reds);
        kind.exec(jobs, args, reds)
    }
}
