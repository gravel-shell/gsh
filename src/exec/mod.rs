mod command;
mod flow;

pub use command::Command;
pub use flow::Flow;

use crate::parse::Line;
use crate::job::SharedJobs;

pub enum Object {
    Flow(Flow),
    Command(Command),
}

impl From<Line> for Object {
    fn from(line: Line) -> Self {
        match line {
            Line::Flow(flow) => Self::Flow(Flow::from(flow)),
            Line::Command(cmd) => Self::Command(Command::from(cmd))
        }
    }
}

impl Object {
    pub fn exec(&self, jobs: &SharedJobs) -> anyhow::Result<()> {
        match self {
            Self::Flow(flow) => flow.exec(jobs),
            Self::Command(cmd) => {
                jobs.with(|jobs| cmd.exec(jobs))?;
                let mut j = jobs.get()?;
                j.wait_fg()?;
                jobs.store(j)?;
                Ok(())
            }
        }
    }
}
