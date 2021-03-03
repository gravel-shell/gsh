mod command;

pub use command::Command;

use crate::parse::Line;
use crate::job::SharedJobs;

pub enum Object {
    Multi(Vec<Object>),
    Single(Command),
}

impl From<Line> for Object {
    fn from(line: Line) -> Self {
        match line {
            Line::Multi(lines) => Self::Multi(lines.into_iter().map(|line| Object::from(line)).collect()),
            Line::Single(cmd) => Self::Single(Command::from(cmd))
        }
    }
}

impl Object {
    pub fn exec(&self, jobs: &SharedJobs) -> anyhow::Result<()> {
        match self {
            Self::Multi(lines) => {
                for line in lines.iter() {
                    line.exec(jobs)?;
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
