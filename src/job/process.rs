use anyhow::Context;
use nix::libc;
use nix::sys::signal::kill;
use nix::unistd::Pid;
use std::convert::TryFrom;
use std::fmt;

use super::{Signal, Status};

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

impl Process {
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
