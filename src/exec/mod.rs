mod command;

pub use command::Command;

use crate::parse::Line;
use crate::job::Jobs;

pub enum Object {
    Command(Command),
}

impl From<Line> for Object {
    fn from(line: Line) -> Self {
        match line {
            Line::Command(cmd) => Self::Command(Command::from(cmd))
        }
    }
}

impl Object {
    pub fn exec(&self, jobs: &mut Jobs) -> anyhow::Result<()> {
        match self {
            Self::Command(cmd) => cmd.exec(jobs),
        }
    }
}
