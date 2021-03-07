use super::{Command, NameSpace};
use crate::job::{SharedJobs, Status};
use crate::parse::{Block as ParseBlk, SpecialStr};

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
    pub fn eval(&self, jobs: &SharedJobs, ns: &mut NameSpace) -> anyhow::Result<()> {
        self.eval_inner(jobs, ns)?;
        Ok(())
    }

    fn eval_inner(&self, jobs: &SharedJobs, ns: &mut NameSpace) -> anyhow::Result<State> {
        match self {
            Self::Single(cmd) => {
                jobs.with(|jobs| cmd.eval(jobs, ns))?;
                let stat = jobs.wait_fg()?;
                if let Some(Status::Exited(code)) = stat {
                    ns.push_var("status", code.to_string());
                }
                Ok(State::Normal)
            }
            Self::Multi(lines) => {
                ns.mark();
                for line in lines.iter() {
                    let state = line.eval_inner(jobs, ns)?;
                    match state {
                        State::Normal => continue,
                        State::Breaked | State::Continued => {
                            ns.drop();
                            return Ok(state);
                        }
                    }
                }
                ns.drop();
                Ok(State::Normal)
            }
            Self::If(cond, first, second) => {
                let cond = matches!(
                    cond.eval_shared(jobs)?.to_lowercase().as_str(),
                    "1" | "y" | "yes" | "true"
                );

                let state = if cond {
                    first.eval_inner(jobs, ns)?
                } else if let Some(sec) = second {
                    sec.eval_inner(jobs, ns)?
                } else {
                    State::Normal
                };

                Ok(state)
            }
            Self::Case(cond, blocks) => {
                let cond = cond.eval_shared(jobs)?;
                for (pats, block) in blocks.iter() {
                    let pats = pats
                        .iter()
                        .map(|pat| pat.eval_shared(jobs))
                        .collect::<Result<Vec<_>, _>>()?;
                    if pats.into_iter().any(|pat| pat == cond) {
                        return Ok(block.eval_inner(jobs, ns)?);
                    }
                }
                Ok(State::Normal)
            }
            Self::For(c, iter, block) => {
                ns.mark();
                for val in iter.eval_shared(jobs)?.split('\n') {
                    ns.push_var(c, val);
                    let state = block.eval_inner(jobs, ns)?;
                    match state {
                        State::Normal | State::Continued => continue,
                        State::Breaked => break,
                    }
                }
                ns.drop();
                Ok(State::Normal)
            }
            Self::While(cond, block) => {
                while matches!(
                    cond.eval_shared(jobs)?.to_lowercase().as_str(),
                    "1" | "y" | "yes" | "true"
                ) {
                    let state = block.eval_inner(jobs, ns)?;
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
