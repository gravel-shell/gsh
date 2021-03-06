use crate::eval::Block;
use crate::job::SharedJobs;
use crate::parse::{parse_line, Parsed};

mod vars;

pub use vars::Vars;

pub struct Session<T> {
    reader: T,
    jobs: SharedJobs,
    vars: Vars,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MoreLine {
    Get(String),
    Eof,
}

pub trait Reader: Sized {
    fn init(jobs: &SharedJobs) -> anyhow::Result<Self>;
    fn next_line(&mut self) -> anyhow::Result<String>;
    fn more_line(&mut self) -> anyhow::Result<MoreLine>;
}

impl<T: Reader> Session<T> {
    pub fn new() -> anyhow::Result<Self> {
        let jobs = SharedJobs::new();
        Ok(Self {
            reader: T::init(&jobs)?,
            jobs,
            vars: Vars::default(),
        })
    }

    pub fn next(&mut self) -> anyhow::Result<bool> {
        let mut line = match self.reader.next_line() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Readline Error: {}", e);
                return Ok(true);
            }
        };

        let line = loop {
            match parse_line(line.as_str()) {
                Ok(Parsed::Complete(cmd)) => break cmd,
                Ok(Parsed::Yet) => {
                    let additional = match self.reader.more_line() {
                        Ok(MoreLine::Get(s)) => s,
                        Ok(MoreLine::Eof) => return Ok(true),
                        Err(e) => {
                            eprintln!("Readline Error: {}", e);
                            return Ok(true);
                        }
                    };
                    line.push('\n');
                    line.push_str(&additional);
                    continue;
                }
                Err(e) => {
                    eprintln!("Parse Error: {}", e);
                    return Ok(true);
                }
            }
        };

        eprintln!("{:?}", line);
        let block = Block::from(line);

        match block.eval(&self.jobs, &mut self.vars) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("{}", e);
                return Ok(true);
            }
        }

        Ok(true)
    }

    pub fn all(&mut self) -> anyhow::Result<()> {
        loop {
            if !self.next()? {
                break;
            }
        }

        Ok(())
    }
}
