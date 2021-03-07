use super::Redirects;

use crate::job::SharedJobs;
use crate::parse::{Arg, Command as ParseCmd, SpecialStr};

use std::process::{Child, Command, Stdio};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct External {
    pub name: SpecialStr,
    pub args: Vec<SpecialStr>,
    pub reds: Redirects,
    pub pipe: Option<Box<External>>,
    pub bg: bool,
}

impl From<ParseCmd> for External {
    fn from(cmd: ParseCmd) -> External {
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

        let reds = Redirects::new(reds);
        let pipe = pipe.map(|pipe| Box::new(Self::from(*pipe)));
        Self {
            name,
            args,
            reds,
            pipe,
            bg,
        }
    }
}

impl External {
    pub fn eval(&self, jobs: &SharedJobs) -> anyhow::Result<()> {
        let child = self.child(jobs, false)?;
        jobs.with(|jobs| {
            if self.bg {
                let (id, pid) = jobs.new_bg(child.id() as i32)?;
                println!("Job %{} ({}) has started.", id, pid);
            } else {
                jobs.new_fg(child.id() as i32)?;
            }
            Ok(())
        })
    }

    pub fn output(&self, jobs: &SharedJobs) -> anyhow::Result<String> {
        let child = self.child(jobs, true)?;
        let output = child.wait_with_output()?;
        Ok(String::from_utf8(output.stdout)?)
    }

    fn child(&self, jobs: &SharedJobs, output: bool) -> anyhow::Result<Child> {
        let mut child = Command::new(&self.name.eval(jobs)?);
        child.args(
            &self
                .args
                .iter()
                .map(|arg| arg.eval(jobs))
                .collect::<Result<Vec<_>, _>>()?,
        );

        let heredoc = self
            .reds
            .redirect(&mut child, jobs, false, output || self.pipe.is_some())?;

        let mut child = child.spawn()?;

        if let Some(s) = heredoc {
            use std::io::Write;
            child.stdin.take().unwrap().write_all(&s)?;
        }

        if let Some(pipe) = &self.pipe {
            pipe.pipe_from(child, jobs, output)
        } else {
            Ok(child)
        }
    }

    fn pipe_from(&self, other: Child, jobs: &SharedJobs, output: bool) -> anyhow::Result<Child> {
        let mut child = Command::new(&self.name.eval(jobs)?);
        child.args(
            &self
                .args
                .iter()
                .map(|arg| arg.eval(jobs))
                .collect::<Result<Vec<_>, _>>()?,
        );

        let heredoc = self
            .reds
            .redirect(&mut child, jobs, true, output || self.pipe.is_some())?;

        child.stdin(Stdio::from(other.stdout.unwrap()));

        let mut child = child.spawn()?;

        if let Some(s) = heredoc {
            use std::io::Write;
            child.stdin.take().unwrap().write_all(&s)?;
        }

        if let Some(pipe) = &self.pipe {
            pipe.pipe_from(child, jobs, output)
        } else {
            Ok(child)
        }
    }
}
