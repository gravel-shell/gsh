use super::RedFile;
use std::fs::{File, OpenOptions};
use std::io;

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

