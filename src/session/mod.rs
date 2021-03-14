mod io;
mod prompt;

pub use io::IOReader;
pub use prompt::PromptReader;

use crate::eval::{Block, NameSpace};
use crate::job::SharedJobs;
use crate::parse::{parse_line, Parsed};

pub struct Session<T> {
    reader: T,
    jobs: SharedJobs,
}

pub trait Reader: Sized {
    #[allow(unused_variables)]
    fn init(&mut self, jobs: &SharedJobs) -> anyhow::Result<()> {
        Ok(())
    }
    fn next_line(&mut self) -> anyhow::Result<Option<String>>;
    fn more_line(&mut self) -> anyhow::Result<Option<String>> {
        self.next_line()
    }
}

impl<T: Reader> Session<T> {
    pub fn new(mut reader: T) -> anyhow::Result<Self> {
        let jobs = SharedJobs::new();
        reader.init(&jobs)?;
        Ok(Self { reader, jobs })
    }

    pub fn next(&mut self, namespace: &mut NameSpace) -> anyhow::Result<bool> {
        let mut line = match self.reader.next_line() {
            Ok(Some(s)) => s,
            Ok(None) => return Ok(false),
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
                        Ok(Some(s)) => s,
                        Ok(None) => return Ok(true),
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

        match block.eval(&self.jobs, namespace) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("{}", e);
                return Ok(true);
            }
        }

        Ok(true)
    }

    pub fn all(&mut self, namespace: &mut NameSpace) -> anyhow::Result<()> {
        loop {
            if !self.next(namespace)? {
                break;
            }
        }

        Ok(())
    }

    pub fn all_with_args<N, A, AS>(
        &mut self,
        namespace: &mut NameSpace,
        name: N,
        args: AS,
    ) -> anyhow::Result<()>
    where
        N: AsRef<str>,
        A: AsRef<str>,
        AS: IntoIterator<Item = A>,
    {
        namespace.mark();
        namespace.set_args(name, args);
        self.all(namespace)?;
        namespace.drop();
        Ok(())
    }
}
