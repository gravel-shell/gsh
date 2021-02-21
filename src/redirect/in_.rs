use super::RedFile;
use std::fs::File;
use std::io;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RedIn {
    Stdin,
    Null,
    File(String),
    HereDoc(String),
}

impl RedIn {
    pub fn from_file(file: RedFile, here_doc: bool) -> anyhow::Result<Self> {
        Ok(match file {
            RedFile::Stdin => Self::Stdin,
            RedFile::Stdout => anyhow::bail!("Can't redirect input from stdout."),
            RedFile::Stderr => anyhow::bail!("Can't redirect input from stderr."),
            RedFile::Null => Self::Null,
            RedFile::File(s) if here_doc => Self::HereDoc(s),
            RedFile::File(s) => Self::File(s),
        })
    }

    pub fn to_reader(self) -> anyhow::Result<RedInReader> {
        Ok(match self {
            Self::Stdin => RedInReader::Stdin(io::stdin()),
            Self::Null => RedInReader::File(File::open("/dev/null")?),
            Self::File(s) => RedInReader::File(File::open(s)?),
            Self::HereDoc(mut s) => {
                s.push('\n');
                RedInReader::Bytes(io::Cursor::new(s.into_bytes()))
            }
        })
    }
}

#[derive(Debug)]
pub enum RedInReader {
    Stdin(io::Stdin),
    File(File),
    Bytes(io::Cursor<Vec<u8>>),
}

impl io::Read for RedInReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Stdin(r) => r.read(buf),
            Self::File(r) => r.read(buf),
            Self::Bytes(r) => r.read(buf),
        }
    }
}
