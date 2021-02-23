use super::Redirects;

use crate::job::Jobs;
use crate::parse::{Arg, Command as ParseCmd};

use std::process::{Child, Command, Stdio};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct External {
    pub name: String,
    pub args: Vec<String>,
    pub reds: Redirects,
    pub pipe: Option<Box<External>>,
}

impl From<ParseCmd> for External {
    fn from(cmd: ParseCmd) -> External {
        let ParseCmd {
            name,
            args: arg_reds,
            pipe,
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

        let reds = Redirects::new(reds);
        let pipe = pipe.map(|pipe| Box::new(Self::from(*pipe)));
        Self {
            name,
            args,
            reds,
            pipe,
        }
    }
}

impl External {
    pub fn exec(&self, jobs: &mut Jobs) -> anyhow::Result<()> {
        let child = self.child()?;
        jobs.new_fg(child.id() as i32)?;
        Ok(())
    }

    fn child(&self) -> anyhow::Result<Child> {
        let mut child = Command::new(&self.name);
        child.args(&self.args);

        let heredoc = self.reds.redirect(&mut child, false, self.pipe.is_some())?;

        let mut child = child.spawn()?;

        if let Some(s) = heredoc {
            use std::io::Write;
            child.stdin.take().unwrap().write_all(s)?;
        }

        if let Some(pipe) = &self.pipe {
            pipe.pipe_from(child)
        } else {
            Ok(child)
        }
    }

    fn pipe_from(&self, other: Child) -> anyhow::Result<Child> {
        let mut child = Command::new(&self.name);
        child.args(&self.args);

        let heredoc = self.reds.redirect(&mut child, true, self.pipe.is_some())?;

        child.stdin(Stdio::from(other.stdout.unwrap()));

        let mut child = child.spawn()?;

        if let Some(s) = heredoc {
            use std::io::Write;
            child.stdin.take().unwrap().write_all(s)?;
        }

        if let Some(pipe) = &self.pipe {
            pipe.pipe_from(child)
        } else {
            Ok(child)
        }
    }
}
