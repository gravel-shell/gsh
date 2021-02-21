use super::RedFile;

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
}
