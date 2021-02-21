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
    pid: Pid,
    suspended: bool,
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

impl Process {
    pub fn new_cmd(name: &str, args: Vec<String>, output: Output) -> anyhow::Result<Self> {
        use crate::redirect::*;
        use std::io::copy;
        use std::process::{Command, Stdio};
        let mut child = Command::new(name);
        child.args(args);

        if output.stdin != RedIn::Stdin {
            child.stdin(Stdio::piped());
        }

        if output.stdout != RedOut::stdout() {
            child.stdout(Stdio::piped());
        }

        if output.stderr != RedOut::stderr() {
            child.stderr(Stdio::piped());
        }

        let child = child
            .spawn()
            .context(format!("Invalid command: {}", name))?;

        let process = Self::from(child.id() as i32);

        let Output {
            stdin,
            stdout,
            stderr,
        } = output;

        if stdin != RedIn::Stdin {
            std::io::copy(&mut stdin.to_reader()?, &mut child.stdin.unwrap())
                .context("Failed to redirect")?;
        }

        match (stdout.kind.clone(), stderr.kind.clone()) {
            (RedOutKind::Stdout, RedOutKind::Stderr) => {}
            (RedOutKind::Stdout, _) => {
                copy(&mut child.stderr.unwrap(), &mut stderr.to_writer()?)
                    .context("Failed to redirect")?;
            }
            (_, RedOutKind::Stderr) => {
                copy(&mut child.stdout.unwrap(), &mut stdout.to_writer()?)
                    .context("Failed to redirect")?;
            }
            (RedOutKind::File(out), RedOutKind::File(err))
                if out == err && stdout.mode == stderr.mode =>
            {
                let mut writer = stdout.to_writer()?;
                copy(&mut child.stdout.unwrap(), &mut writer).context("Failed to redirect")?;
                copy(&mut child.stderr.unwrap(), &mut writer).context("Failed to redirect")?;
            }
            (_, _) => {
                copy(&mut child.stderr.unwrap(), &mut stderr.to_writer()?)
                    .context("Failed to redirect")?;
                copy(&mut child.stdout.unwrap(), &mut stdout.to_writer()?)
                    .context("Failed to redirect")?;
            }
        };

        Ok(process)
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
