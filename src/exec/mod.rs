mod command;

pub use command::Command;

use crate::parse::Line;
use crate::job::SharedJobs;

pub enum Object {
    Multi(Vec<Command>),
    Single(Command),
}

impl From<Line> for Object {
    fn from(line: Line) -> Self {
        match line {
            Line::Multi(cmds) => Self::Multi(cmds.into_iter().map(|cmd| Command::from(cmd)).collect()),
            Line::Single(cmd) => Self::Single(Command::from(cmd))
        }
    }
}

impl Object {
    pub fn exec(&self, jobs: &SharedJobs) -> anyhow::Result<()> {
        match self {
            Self::Multi(cmds) => {
                for cmd in cmds.iter() {
                    jobs.with(|jobs| cmd.exec(jobs))?;
                    jobs.wait_fg()?;
                }
                Ok(())
            },
            Self::Single(cmd) => {
                jobs.with(|jobs| cmd.exec(jobs))?;
                jobs.wait_fg()?;
                Ok(())
            }
        }
    }
}
