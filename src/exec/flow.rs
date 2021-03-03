use super::Command;
use crate::parse::Flow as ParseFlow;
use crate::job::SharedJobs;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Flow {
    Bracket(Vec<Command>),
}

impl From<ParseFlow> for Flow {
    fn from(flow: ParseFlow) -> Self {
        match flow {
            ParseFlow::Bracket(cmds) => {
                Self::Bracket(cmds.into_iter().map(|cmd| Command::from(cmd)).collect())
            }
        }
    }
}

impl Flow {
    pub fn exec(&self, shared_jobs: &SharedJobs) -> anyhow::Result<()> {
        match self {
            Self::Bracket(cmds) => {
                for cmd in cmds.iter() {
                    shared_jobs.with(|jobs| cmd.exec(jobs))?;
                    let mut jobs = shared_jobs.get()?;
                    jobs.wait_fg()?;
                    shared_jobs.store(jobs)?;
                }
                Ok(())
            }
        }
    }
}
