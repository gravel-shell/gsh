mod command;

pub use command::Command;

use crate::job::{SharedJobs, Status};
use crate::parse::{Line, SpecialStr};
use crate::session::Vars;

pub enum Eval {
    Single(Command),
    Multi(Vec<Eval>),
    If(SpecialStr, Box<Eval>, Option<Box<Eval>>),
    Break,
    Continue,
}

impl From<Line> for Eval {
    fn from(line: Line) -> Self {
        match line {
            Line::Multi(lines) => {
                Self::Multi(lines.into_iter().map(|line| Eval::from(line)).collect())
            }
            Line::Single(cmd) => Self::Single(Command::from(cmd)),
            Line::If(cond, first, second) => Self::If(
                cond,
                Box::new(Eval::from(*first)),
                second.map(|sec| Box::new(Eval::from(*sec))),
            ),
            Line::Break => Self::Break,
            Line::Continue => Self::Continue,
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
            Self::If(cond, first, second) => {
                let cond = matches!(
                    cond.eval()?.to_lowercase().as_str(),
                    "0" | "y" | "yes" | "true"
                );

                if cond {
                    first.eval(jobs, vars)?;
                } else if let Some(sec) = second {
                    sec.eval(jobs, vars)?;
                }

                Ok(())
            }
            Self::Break => Ok(()),
            Self::Continue => Ok(()),
        }
    }
}
