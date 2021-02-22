use super::RedFile;

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
}
