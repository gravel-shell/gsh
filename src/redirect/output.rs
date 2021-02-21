use super::{RedIn, RedKind, RedOut, Redirect};

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
                    RedKind::Stdin => res.stdin = RedIn::from_file(red.file, false)?,
                    RedKind::HereDoc => res.stdin = RedIn::from_file(red.file, true)?,
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
