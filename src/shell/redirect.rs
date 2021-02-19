use std::fs::{File, OpenOptions};
use std::io;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Redirect {
    pub kind: RedKind,
    pub file: RedFile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedKind {
    Stdin,
    Stdout(RedOutMode),
    Stderr(RedOutMode),
    Bind(RedOutMode),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RedFile {
    Stdin,
    Stdout,
    Stderr,
    Null,
    File(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedOut {
    pub kind: RedOutKind,
    pub mode: RedOutMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedOutMode {
    Overwrite,
    Append,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RedOutKind {
    Stdout,
    Stderr,
    Null,
    File(String),
}

impl RedOut {
    pub fn stdout() -> Self {
        Self {
            kind: RedOutKind::Stdout,
            mode: RedOutMode::Overwrite,
        }
    }

    pub fn stderr() -> Self {
        Self {
            kind: RedOutKind::Stderr,
            mode: RedOutMode::Overwrite,
        }
    }

    pub fn from_file(file: RedFile, mode: RedOutMode) -> anyhow::Result<Self> {
        Ok(match file {
            RedFile::Stdin => anyhow::bail!("Can't redirect output to stdin."),
            RedFile::Stdout => Self {
                kind: RedOutKind::Stdout,
                mode,
            },
            RedFile::Stderr => Self {
                kind: RedOutKind::Stderr,
                mode,
            },
            RedFile::Null => Self {
                kind: RedOutKind::Null,
                mode,
            },
            RedFile::File(s) => Self {
                kind: RedOutKind::File(s),
                mode,
            },
        })
    }

    pub fn to_writer(self) -> anyhow::Result<RedOutWriter> {
        let RedOut { kind, mode } = self;

        let mut option = OpenOptions::new();
        match mode {
            RedOutMode::Overwrite => option.write(true).create(true),
            RedOutMode::Append => option.write(true).append(true),
        };

        Ok(match kind {
            RedOutKind::Stdout => RedOutWriter::Stdout(io::stdout()),
            RedOutKind::Stderr => RedOutWriter::Stderr(io::stderr()),
            RedOutKind::Null => RedOutWriter::File(option.open("/dev/null")?),
            RedOutKind::File(s) => RedOutWriter::File(if std::path::Path::new(&s).exists() {
                option.open(s)
            } else {
                File::create(s)
            }?),
        })
    }
}

#[derive(Debug)]
pub enum RedOutWriter {
    Stdout(io::Stdout),
    Stderr(io::Stderr),
    File(File),
}

impl io::Write for RedOutWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Self::Stdout(w) => w.write(buf),
            Self::Stderr(w) => w.write(buf),
            Self::File(w) => w.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Self::Stdout(w) => w.flush(),
            Self::Stderr(w) => w.flush(),
            Self::File(w) => w.flush(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RedIn {
    Stdin,
    Null,
    File(String),
}

impl RedIn {
    pub fn from_file(file: RedFile) -> anyhow::Result<Self> {
        Ok(match file {
            RedFile::Stdin => Self::Stdin,
            RedFile::Stdout => anyhow::bail!("Can't redirect input from stdout."),
            RedFile::Stderr => anyhow::bail!("Can't redirect input from stderr."),
            RedFile::Null => Self::Null,
            RedFile::File(s) => Self::File(s),
        })
    }

    pub fn to_reader(self) -> anyhow::Result<RedInReader> {
        Ok(match self {
            Self::Stdin => RedInReader::Stdin(io::stdin()),
            Self::Null => RedInReader::File(File::open("/dev/null")?),
            Self::File(s) => RedInReader::File(File::open(s)?),
        })
    }
}

#[derive(Debug)]
pub enum RedInReader {
    Stdin(io::Stdin),
    File(File),
}

impl io::Read for RedInReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Stdin(r) => r.read(buf),
            Self::File(r) => r.read(buf),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    pub stdin: RedIn,
    pub stdout: RedOut,
    pub stderr: RedOut,
}

impl Output {
    pub fn from<T: IntoIterator<Item = Redirect>>(reds: T) -> anyhow::Result<Self> {
        let res = Self {
            stdin: RedIn::Stdin,
            stdout: RedOut::stdout(),
            stderr: RedOut::stderr(),
        };
        reds.into_iter()
            .fold(Ok(res), |acc: anyhow::Result<_>, red| {
                let mut res = acc?;
                match red.kind {
                    RedKind::Stdin => res.stdin = RedIn::from_file(red.file)?,
                    RedKind::Stdout(m) => res.stdout = RedOut::from_file(red.file, m)?,
                    RedKind::Stderr(m) => res.stderr = RedOut::from_file(red.file, m)?,
                    RedKind::Bind(m) => {
                        res.stdout = RedOut::from_file(red.file.clone(), m)?;
                        res.stderr = RedOut::from_file(red.file, m)?;
                    }
                }
                Ok(res)
            })
    }
}
