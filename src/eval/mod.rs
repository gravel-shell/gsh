mod command;

pub use command::Command;

use crate::job::{SharedJobs, Status};
use crate::parse::{Block as ParseBlk, SpecialStr};
use crate::session::Vars;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Block {
    Single(Command),
    Multi(Vec<Block>),
    If(SpecialStr, Box<Block>, Option<Box<Block>>),
    Case(SpecialStr, Vec<(Vec<SpecialStr>, Block)>),
    For(String, SpecialStr, Box<Block>),
    While(SpecialStr, Box<Block>),
    Break,
    Continue,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Normal,
    Breaked,
    Continued,
}

impl From<ParseBlk> for Block {
    fn from(block: ParseBlk) -> Self {
        match block {
            ParseBlk::Multi(blocks) => {
                Self::Multi(blocks.into_iter().map(|block| Self::from(block)).collect())
            }
            ParseBlk::Single(cmd) => Self::Single(Command::from(cmd)),
            ParseBlk::If(cond, first, second) => Self::If(
                cond,
                Box::new(Self::from(*first)),
                second.map(|sec| Box::new(Self::from(*sec))),
            ),
            ParseBlk::Case(cond, blocks) => Self::Case(
                cond,
                blocks
                    .into_iter()
                    .map(|(pats, block)| (pats, Self::from(block)))
                    .collect(),
            ),
            ParseBlk::For(c, iter, block) => Self::For(c, iter, Box::new(Self::from(*block))),
            ParseBlk::While(cond, block) => Self::While(cond, Box::new(Self::from(*block))),
            ParseBlk::Break => Self::Break,
            ParseBlk::Continue => Self::Continue,
        }
    }
}

impl Block {
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
                }
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
                    "1" | "y" | "yes" | "true"
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
            Self::Case(cond, blocks) => {
                let cond = cond.eval()?;
                for (pats, block) in blocks.iter() {
                    let pats = pats
                        .iter()
                        .map(|pat| pat.eval())
                        .collect::<Result<Vec<_>, _>>()?;
                    if pats.into_iter().any(|pat| pat == cond) {
                        return Ok(block.eval_inner(jobs, vars)?);
                    }
                }
                Ok(State::Normal)
            }
            Self::For(c, iter, block) => {
                for val in iter.eval()?.split('\n') {
                    std::env::set_var(c, val);
                    let state = block.eval_inner(jobs, vars)?;
                    match state {
                        State::Normal | State::Continued => continue,
                        State::Breaked => break,
                    }
                }
                std::env::remove_var(c);
                Ok(State::Normal)
            }
            Self::While(cond, block) => {
                while matches!(
                    cond.eval()?.to_lowercase().as_str(),
                    "1" | "y" | "yes" | "true"
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
