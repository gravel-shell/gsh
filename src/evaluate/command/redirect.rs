use crate::parse::{RedKind, RedTarget, Redirect, SpecialStr};
use std::fs::{File, OpenOptions};
use std::process::{Command, Stdio};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Redirects(RedirectsInner);

impl Redirects {
    pub fn new(reds: Vec<Redirect>) -> Self {
        Self(RedirectsInner::new(reds))
    }

    pub fn redirect(
        &self,
        cmd: &mut Command,
        piped_in: bool,
        piped_out: bool,
    ) -> anyhow::Result<Option<Vec<u8>>> {
        self.0.redirect(cmd, piped_in, piped_out)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum RedirectsInner {
    Each(Option<RedIn>, Option<RedOut>, Option<RedOut>),
    Bind(Option<RedIn>, Option<RedOut>),
}

fn target2str(target: RedTarget) -> SpecialStr {
    match target {
        RedTarget::Stdin => SpecialStr::from(String::from("/dev/stdin")),
        RedTarget::Stdout => SpecialStr::from(String::from("/dev/stdout")),
        RedTarget::Stderr => SpecialStr::from(String::from("/dev/stderr")),
        RedTarget::Null => SpecialStr::from(String::from("/dev/null")),
        RedTarget::Other(s) => s,
    }
}

impl RedirectsInner {
    fn new(reds: Vec<Redirect>) -> Self {
        let mut stdout = None;
        let mut stderr = None;
        let mut stdin = None;
        for red in reds {
            match red.kind {
                RedKind::OverwriteStdout => {
                    stdout = Some(RedOut::overwrite(red.target));
                }
                RedKind::AppendStdout => {
                    stdout = Some(RedOut::append(red.target));
                }
                RedKind::OverwriteStderr => {
                    stderr = Some(RedOut::overwrite(red.target));
                }
                RedKind::AppendStderr => {
                    stderr = Some(RedOut::append(red.target));
                }
                RedKind::OverwriteBoth => {
                    stdout = Some(RedOut::overwrite(red.target));
                    stderr = stdout.clone();
                }
                RedKind::AppendBoth => {
                    stdout = Some(RedOut::append(red.target));
                    stderr = stdout.clone();
                }
                RedKind::Stdin => {
                    stdin = Some(RedIn {
                        mode: InMode::Normal,
                        target: target2str(red.target),
                    });
                }
                RedKind::HereDoc => {
                    stdin = Some(RedIn {
                        mode: InMode::HereDoc,
                        target: target2str(red.target),
                    });
                }
            }
        }

        if stdout == stderr {
            Self::Bind(stdin, stdout)
        } else {
            Self::Each(stdin, stdout, stderr)
        }
    }

    fn redirect(
        &self,
        cmd: &mut Command,
        piped_in: bool,
        piped_out: bool,
    ) -> anyhow::Result<Option<Vec<u8>>> {
        let stdin = match self {
            Self::Bind(stdin, Some(stdout)) if piped_out => {
                let err = stdout.mode.option().open(&stdout.target.eval()?)?;
                cmd.stdout(Stdio::piped());
                cmd.stderr(Stdio::from(err));
                stdin
            }
            Self::Bind(stdin, Some(stdout)) => {
                let out = stdout.mode.option().open(&stdout.target.eval()?)?;
                let err = out.try_clone()?;
                cmd.stdout(Stdio::from(out));
                cmd.stderr(Stdio::from(err));
                stdin
            }
            Self::Bind(stdin, None) if piped_out => {
                cmd.stdout(Stdio::piped());
                stdin
            }
            Self::Bind(stdin, None) => stdin,
            Self::Each(stdin, stdout, stderr) => {
                if piped_out {
                    cmd.stdout(Stdio::piped());
                } else if let Some(stdout) = stdout {
                    let out = stdout.mode.option().open(&stdout.target.eval()?)?;
                    cmd.stdout(Stdio::from(out));
                }

                if let Some(stderr) = stderr {
                    let err = stderr.mode.option().open(&stderr.target.eval()?)?;
                    cmd.stderr(Stdio::from(err));
                }

                stdin
            }
        };

        let mut s = None;
        if piped_in {
            cmd.stdin(Stdio::piped());
        } else if let Some(stdin) = stdin {
            let target = stdin.target.eval()?;
            match stdin.mode {
                InMode::Normal => {
                    cmd.stdin(Stdio::from(File::open(&target)?));
                }
                InMode::HereDoc => {
                    cmd.stdin(Stdio::piped());
                    s = Some(target.into_bytes());
                }
            }
        }

        Ok(s)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RedOut {
    mode: OutMode,
    target: SpecialStr,
}

impl RedOut {
    fn overwrite(target: RedTarget) -> Self {
        Self {
            mode: OutMode::Overwrite,
            target: target2str(target),
        }
    }

    fn append(target: RedTarget) -> Self {
        Self {
            mode: OutMode::Append,
            target: target2str(target),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OutMode {
    Overwrite,
    Append,
}

impl OutMode {
    fn option(&self) -> OpenOptions {
        let mut opt = OpenOptions::new();
        match self {
            Self::Overwrite => opt.write(true).create(true),
            Self::Append => opt.write(true).append(true),
        };
        opt
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RedIn {
    mode: InMode,
    target: SpecialStr,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InMode {
    Normal,
    HereDoc,
}
