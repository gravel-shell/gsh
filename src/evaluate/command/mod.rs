mod builtin;
mod external;
mod redirect;

pub use builtin::{Builtin, BuiltinKind};
pub use external::External;
pub use redirect::Redirects;

use crate::job::Jobs;
use crate::parse::{Arg, Command as ParseCmd};
use crate::session::Vars;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Builtin(Builtin),
    External(External),
}

impl From<ParseCmd> for Command {
    fn from(cmd: ParseCmd) -> Self {
        let ParseCmd {
            name,
            args: arg_reds,
            pipe,
            bg,
        } = cmd;

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

        let kind = BuiltinKind::new(&name);
        if let Some(kind) = kind {
            return Self::Builtin(Builtin::new(kind, args));
        }

        let reds = Redirects::new(reds);
        let pipe = pipe.map(|pipe| Box::new(External::from(*pipe)));
        Self::External(External {
            name,
            args,
            reds,
            pipe,
            bg,
        })
    }
}

impl Command {
    pub fn eval(&self, jobs: &mut Jobs, vars: &mut Vars) -> anyhow::Result<()> {
        match self {
            Self::Builtin(ref builtin) => builtin.eval(jobs, vars),
            Self::External(ref external) => external.eval(jobs),
        }
    }
}
