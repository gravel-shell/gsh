mod command;

pub use command::Command;

use crate::parse::Line;
use crate::job::{SharedJobs, Status};
use crate::session::Vars;

pub enum Eval {
    Multi(Vec<Eval>),
    Single(Command),
}

impl From<Line> for Eval {
    fn from(line: Line) -> Self {
        match line {
            Line::Multi(lines) => Self::Multi(lines.into_iter().map(|line| Eval::from(line)).collect()),
            Line::Single(cmd) => Self::Single(Command::from(cmd))
        }
    }
}

impl Eval {
    pub fn eval(&self, jobs: &SharedJobs, vars: &mut Vars) -> anyhow::Result<()> {
        match self {
            Self::Multi(lines) => {
                vars.mark();
                for line in lines.iter() {
                    line.eval(jobs, vars)?;
                }
                vars.drop();
                Ok(())
            },
            Self::Single(cmd) => {
                jobs.with(|jobs| cmd.eval(jobs, vars))?;
                let stat = jobs.wait_fg()?;
                if let Some(Status::Exited(code)) = stat {
                    vars.push("status", code.to_string());
                }
                Ok(())
            }
        }
    }
}
