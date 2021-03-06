mod builtin;
mod external;
mod redirect;

pub use builtin::{Builtin, BuiltinKind};
pub use external::External;
pub use redirect::Redirects;

use crate::job::Jobs;
use crate::parse::Command as ParseCmd;
use super::NameSpace;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command(External);

impl From<ParseCmd> for Command {
    fn from(cmd: ParseCmd) -> Self {
        Self(External::from(cmd))
    }
}

impl Command {
    pub fn eval(&self, jobs: &mut Jobs, ns: &mut NameSpace) -> anyhow::Result<()> {
        let kind = BuiltinKind::new(self.0.name.eval()?);

        if let Some(kind) = kind {
            Builtin::new(
                kind,
                self.0
                    .args
                    .iter()
                    .map(|arg| arg.eval())
                    .collect::<Result<Vec<_>, _>>()?,
            )
            .eval(jobs, ns)
        } else {
            self.0.eval(jobs)
        }
    }

    pub fn output(&self) -> anyhow::Result<String> {
        self.0.output()
    }
}
