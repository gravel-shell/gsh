mod command;

pub use command::Command;

use crate::job::{SharedJobs, Status};
use crate::parse::Line;
use crate::session::Vars;

pub enum Eval {
    Single(Command),
    Multi(Vec<Eval>),
    If(Option<Box<Eval>>),
}

impl From<Line> for Eval {
    fn from(line: Line) -> Self {
        match line {
            Line::Multi(lines) => {
                Self::Multi(lines.into_iter().map(|line| Eval::from(line)).collect())
            }
            Line::Single(cmd) => Self::Single(Command::from(cmd)),
            Line::If(cond, first, second) => Eval::If(if cond {
                Some(Box::new(Eval::from(*first)))
            } else if let Some(sec) = second {
                Some(Box::new(Eval::from(*sec)))
            } else {
                None
            })
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
            }
            Self::Single(cmd) => {
                jobs.with(|jobs| cmd.eval(jobs, vars))?;
                let stat = jobs.wait_fg()?;
                if let Some(Status::Exited(code)) = stat {
                    vars.push("status", code.to_string());
                }
                Ok(())
            }
            Self::If(Some(eval)) => eval.eval(jobs, vars),
            Self::If(None) => Ok(())
        }
    }
}
