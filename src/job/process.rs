use anyhow::Context;
use nix::libc;
use nix::sys::signal::kill;
use nix::unistd::Pid;
use std::convert::TryFrom;
use std::fmt;

use super::{Signal, Status};
use crate::redirect::Output;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Process {
    pub(super) pid: Pid,
    pub(super) suspended: bool,
}

impl fmt::Display for Process {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "pid: {} (suspended: {})", self.pid, self.suspended)
    }
}

impl std::str::FromStr for Process {
    type Err = std::num::ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.parse::<i32>()?.into())
    }
}

impl From<i32> for Process {
    fn from(id: i32) -> Self {
        Self {
            pid: Pid::from_raw(id),
            suspended: false,
        }
    }
}

impl Into<i32> for Process {
    fn into(self) -> i32 {
        self.pid.as_raw()
    }
}

impl From<Pid> for Process {
    fn from(id: Pid) -> Self {
        Self {
            pid: id,
            suspended: false,
        }
    }
}

impl Into<Pid> for Process {
    fn into(self) -> Pid {
        self.pid
    }
}

fn option(mode: crate::redirect::RedOutMode) -> std::fs::OpenOptions {
    use crate::redirect::RedOutMode;
    let mut option = std::fs::OpenOptions::new();
    match mode {
        RedOutMode::Overwrite => option.write(true).create(true),
        RedOutMode::Append => option.write(true).append(true),
    };
    option
}

impl Process {
    pub fn new_cmd(name: &str, args: Vec<String>, output: Output) -> anyhow::Result<Self> {
        use crate::redirect::*;
        use std::fs::File;
        use std::io::copy;
        use std::process::{Command, Stdio};
        let mut child = Command::new(name);
        child.args(args);

        let Output {
            stdin,
            stdout,
            stderr,
        } = output;

        match stdin {
            RedIn::Stdin => {}
            RedIn::Null => {
                child.stdin(Stdio::from(File::open("/dev/null")?));
            }
            RedIn::File(ref s) => {
                child.stdin(Stdio::from(File::open(s)?));
            }
            RedIn::HereDoc(_) => {
                child.stdin(Stdio::piped());
            }
        }

        if matches!(stdout.kind, RedOutKind::Null | RedOutKind::File(_)) && stdout == stderr {
            let file = match stdout.kind {
                RedOutKind::Null => "/dev/null",
                RedOutKind::File(ref s) => s,
                _ => unreachable!(),
            };

            let out = option(stdout.mode).open(file)?;
            let err = out.try_clone()?;

            child.stdout(Stdio::from(out));
            child.stderr(Stdio::from(err));
        } else {
            match stdout.kind {
                RedOutKind::Stdout => {}
                RedOutKind::Stderr => {
                    child.stdout(Stdio::piped());
                }
                RedOutKind::Null => {
                    child.stdout(Stdio::from(option(stdout.mode).open("/dev/null")?));
                }
                RedOutKind::File(ref s) => {
                    child.stdout(Stdio::from(option(stdout.mode).open(s)?));
                }
            }

            match stderr.kind {
                RedOutKind::Stdout => {
                    child.stderr(Stdio::piped());
                }
                RedOutKind::Stderr => {}
                RedOutKind::Null => {
                    child.stderr(Stdio::from(option(stderr.mode).open("/dev/null")?));
                }
                RedOutKind::File(ref s) => {
                    child.stderr(Stdio::from(option(stderr.mode).open(s)?));
                }
            }
        }

        let child = child
            .spawn()
            .context(format!("Invalid command: {}", name))?;

        let process = Self::from(child.id() as i32);

        if let RedIn::HereDoc(s) = stdin {
            use std::io::Write;
            child.stdin.unwrap().write_all(s.as_bytes())?;
        }

        if let RedOutKind::Stderr = stdout.kind {
            copy(&mut child.stdout.unwrap(), &mut std::io::stderr())?;
        }

        if let RedOutKind::Stdout = stderr.kind {
            copy(&mut child.stderr.unwrap(), &mut std::io::stdout())?;
        }

        Ok(process)
    }

    pub fn pid(&self) -> i32 {
        self.pid.as_raw()
    }

    pub fn suspended(&self) -> bool {
        self.suspended
    }

    pub fn interrupt(self) -> anyhow::Result<Status> {
        kill(self.into(), Signal::SIGINT).context("Failed to inetrrupt the process.")?;
        Ok(Status::Signaled(Signal::SIGINT))
    }

    pub fn suspend(&mut self) -> anyhow::Result<Status> {
        if self.suspended {
            anyhow::bail!("The process is already suspended.");
        }
        self.suspended = true;
        kill((*self).into(), Signal::SIGSTOP).context("Failed to inetrrupt the process.")?;
        Ok(Status::Signaled(Signal::SIGSTOP))
    }

    pub fn restart(&mut self) -> anyhow::Result<Status> {
        if !self.suspended {
            anyhow::bail!("The process is not suspended.");
        }
        self.suspended = false;
        kill((*self).into(), Signal::SIGCONT).context("Failed to inetrrupt the process.")?;
        Ok(Status::Signaled(Signal::SIGCONT))
    }

    pub fn wait(&self) -> anyhow::Result<Status> {
        let (code, status, is_error) = unsafe {
            let mut siginfo = std::mem::zeroed();
            let error = libc::waitid(
                libc::P_PID,
                self.pid.as_raw() as u32,
                &mut siginfo,
                libc::WEXITED | libc::WSTOPPED,
            );
            let siginfo = siginfo as libc::siginfo_t;
            (
                siginfo.si_code as i32,
                siginfo.si_status() as i32,
                error == -1,
            )
        };

        if is_error {
            Err(nix::Error::Sys(nix::errno::Errno::last()))
                .context("Failed to wait the process.")?;
        }

        Ok(match code {
            libc::CLD_EXITED => Status::Exited(status),
            _ => Status::Signaled(Signal::try_from(status).context("Unnexpected signal.")?),
        })
    }
}

pub fn sigchld() -> anyhow::Result<Option<(i32, Status)>> {
    let (pid, code, status, is_error) = unsafe {
        let mut siginfo = std::mem::zeroed();
        let error = libc::waitid(
            libc::P_ALL,
            0,
            &mut siginfo,
            libc::WEXITED | libc::WSTOPPED | libc::WCONTINUED | libc::WNOWAIT,
        );
        let siginfo = siginfo as libc::siginfo_t;
        (
            siginfo.si_pid() as i32,
            siginfo.si_code as i32,
            siginfo.si_status() as i32,
            error == -1,
        )
    };

    if is_error {
        return Ok(None);
    }

    let status = match code {
        libc::CLD_EXITED => Status::Exited(status),
        _ => Status::Signaled(Signal::try_from(status).context("Unnexpected signal.")?),
    };

    Ok(Some((pid, status)))
}
