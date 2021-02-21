use crate::job::CurPid;
use crate::parse::{Parsed, parse_line};

pub struct Session<T> {
    reader: T,
    cur_pid: CurPid,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MoreLine {
    Get(String),
    Eof,
}

pub trait Reader: Sized {
    fn init(cur_pid: &CurPid) -> anyhow::Result<Self>;
    fn next_line(&mut self) -> anyhow::Result<String>;
    fn more_line(&mut self) -> anyhow::Result<MoreLine>;
}

impl<T: Reader> Session<T> {
    pub fn new() -> anyhow::Result<Self> {
        let cur_pid = CurPid::new();
        Ok(Self {
            reader: T::init(&cur_pid)?,
            cur_pid,
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

        let cmd = loop {
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

        let id = match cmd.exec() {
            Ok(Some(id)) => id,
            Ok(None) => return Ok(true),
            Err(e) => {
                eprintln!("{}", e);
                return Ok(true);
            }
        };

        self.cur_pid.store(id)?;
        id.wait()?;
        self.cur_pid.reset()?;

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
