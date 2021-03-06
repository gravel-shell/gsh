mod command;

pub use command::Command;

use crate::job::{SharedJobs, Status};
use crate::parse::{Line, SpecialStr};
use crate::session::Vars;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Eval {
    Single(Command),
    Multi(Vec<Eval>),
    If(SpecialStr, Box<Eval>, Option<Box<Eval>>),
    While(SpecialStr, Box<Eval>),
    Break,
    Continue,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Normal,
    Breaked,
    Continued,
}

impl From<Line> for Eval {
    fn from(line: Line) -> Self {
        match line {
            Line::Multi(lines) => {
                Self::Multi(lines.into_iter().map(|line| Self::from(line)).collect())
            }
            Line::Single(cmd) => Self::Single(Command::from(cmd)),
            Line::If(cond, first, second) => Self::If(
                cond,
                Box::new(Self::from(*first)),
                second.map(|sec| Box::new(Self::from(*sec))),
            ),
            Line::While(cond, block) => Self::While(cond, Box::new(Self::from(*block))),
            Line::Break => Self::Break,
            Line::Continue => Self::Continue,
        }
    }
}

impl Eval {
    pub fn eval(&self, jobs: &SharedJobs, vars: &mut Vars) -> anyhow::Result<()> {
        self.eval_inner(jobs, vars)?;
        Ok(())
    }

    fn eval_inner(&self, jobs: &SharedJobs, vars: &mut Vars) -> anyhow::Result<State> {
        match self {
            Self::Multi(lines) => {
                vars.mark();
                for line in lines.iter() {
                    let state = line.eval_inner(jobs, vars)?;
                    match state {
                        State::Normal => continue,
                        State::Breaked | State::Continued => {
                            vars.drop();
                            return Ok(state);
                        }
                    }
                };
                vars.drop();
                Ok(State::Normal)
            }
            Self::Single(cmd) => {
                jobs.with(|jobs| cmd.eval(jobs, vars))?;
                let stat = jobs.wait_fg()?;
                if let Some(Status::Exited(code)) = stat {
                    vars.push("status", code.to_string());
                }
                Ok(State::Normal)
            }
            Self::If(cond, first, second) => {
                let cond = matches!(
                    cond.eval()?.to_lowercase().as_str(),
                    "0" | "y" | "yes" | "true"
                );

                let state = if cond {
                    first.eval_inner(jobs, vars)?
                } else if let Some(sec) = second {
                    sec.eval_inner(jobs, vars)?
                } else {
                    State::Normal
                };

                Ok(state)
            }
            Self::While(cond, block) => {
                while matches!(
                        cond.eval()?.to_lowercase().as_str(),
                        "0" | "y" | "yes" | "true"
                ) {
                    let state = block.eval_inner(jobs, vars)?;
                    match state {
                        State::Normal | State::Continued => continue,
                        State::Breaked => break,
                    }
                }
                Ok(State::Normal)
            }
            Self::Break => Ok(State::Breaked),
            Self::Continue => Ok(State::Continued),
        }
    }
}
