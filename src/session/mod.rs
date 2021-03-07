mod prompt;

pub use prompt::PromptReader;

use crate::eval::{Block, NameSpace};
use crate::job::SharedJobs;
use crate::parse::{parse_line, Parsed};

pub struct Session<T> {
    reader: T,
    jobs: SharedJobs,
    namespace: NameSpace,
}

pub trait Reader: Sized {
    fn init(&mut self, jobs: &SharedJobs) -> anyhow::Result<()>;
    fn next_line(&mut self) -> anyhow::Result<Option<String>>;
    fn more_line(&mut self) -> anyhow::Result<Option<String>> {
        self.next_line()
    }
}

impl<T: Reader> Session<T> {
    pub fn new(mut reader: T) -> anyhow::Result<Self> {
        let jobs = SharedJobs::new();
        reader.init(&jobs)?;
        Ok(Self {
            reader,
            jobs,
            namespace: NameSpace::default(),
        })
    }

    pub fn next(&mut self) -> anyhow::Result<bool> {
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

        match block.eval(&self.jobs, &mut self.namespace) {
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

    pub fn source<R: Reader>(&mut self, mut reader: R) -> anyhow::Result<()> {
        reader.init(&self.jobs)?;
        let mut session = Session::<R> {
            reader,
            jobs: self.jobs.clone(),
            namespace: self.namespace.clone(),
        };
        session.all()?;
        self.namespace = session.namespace;
        Ok(())
    }

    pub fn source_with_args<R: Reader, N, A, AS>(
        &mut self,
        mut reader: R,
        name: N,
        args: AS,
    ) -> anyhow::Result<()>
    where
        N: AsRef<str>,
        A: AsRef<str>,
        AS: IntoIterator<Item = A>,
    {
        reader.init(&self.jobs)?;
        let mut session = Session::<R> {
            reader,
            jobs: self.jobs.clone(),
            namespace: self.namespace.clone(),
        };
        session.all_with_args(name, args)?;
        self.namespace = session.namespace;
        Ok(())
    }

    pub fn all_with_args<N, A, AS>(&mut self, name: N, args: AS) -> anyhow::Result<()>
    where
        N: AsRef<str>,
        A: AsRef<str>,
        AS: IntoIterator<Item = A>,
    {
        self.namespace.mark();
        self.namespace.set_args(name, args);
        self.all()?;
        self.namespace.drop();
        Ok(())
    }
}
