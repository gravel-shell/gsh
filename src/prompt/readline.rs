extern crate rustyline;

use rustyline::{error::ReadlineError, Editor};

#[derive(Debug)]
pub struct Reader(Editor<()>);

impl Reader {
    pub fn new() -> Self {
        Self(Editor::<()>::new())
    }

    pub fn read(&mut self) -> anyhow::Result<String> {
        match self.0.readline(" $") {
            Ok(s) => Ok(s),
            Err(ReadlineError::Interrupted) => Ok(String::new()),
            Err(ReadlineError::Eof) => Ok(String::from("exit")),
            Err(e) => Err(e)?,
        }
    }
}
