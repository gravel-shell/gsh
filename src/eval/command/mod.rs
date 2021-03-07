mod builtin;
mod external;
mod redirect;

pub use builtin::{Builtin, BuiltinKind};
pub use external::External;
pub use redirect::Redirects;

use super::NameSpace;
use crate::job::SharedJobs;
use crate::parse::Command as ParseCmd;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command(External);

impl From<ParseCmd> for Command {
    fn from(cmd: ParseCmd) -> Self {
        Self(External::from(cmd))
    }
}

impl Command {
    pub fn eval(&self, jobs: &SharedJobs, ns: &mut NameSpace) -> anyhow::Result<()> {
        let name = self.0.name.eval(jobs)?;
        let proc = ns.get_proc(&name);
        if let Some(proc) = proc {
            return proc.eval_with_args(&name, self.0.args.eval(jobs)?, jobs, ns);
        }

        let kind = BuiltinKind::new(name);
        if let Some(kind) = kind {
            return Builtin::new(
                kind,
                self.0.args.eval(jobs)?,
            )
            .eval(jobs, ns);
        }

        self.0.eval(jobs)
    }

    pub fn output(&self, jobs: &SharedJobs) -> anyhow::Result<String> {
        self.0.output(jobs)
    }
}
