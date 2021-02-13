//! Raw API bindings.
extern crate nix;

use anyhow::Context;
use nix::libc;
use nix::sys::signal::kill;
use nix::unistd::Pid as NixPid;
use std::convert::TryFrom;
use std::fmt;

pub use nix::sys::signal::Signal;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Pid(i32);

impl fmt::Display for Pid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for Pid {
    type Err = std::num::ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.parse::<i32>()?.into())
    }
}

impl From<i32> for Pid {
    fn from(id: i32) -> Self {
        Self(id)
    }
}

impl Into<i32> for Pid {
    fn into(self) -> i32 {
        self.0
    }
}

impl From<NixPid> for Pid {
    fn from(id: NixPid) -> Self {
        Self(id.as_raw())
    }
}

impl Into<NixPid> for Pid {
    fn into(self) -> NixPid {
        NixPid::from_raw(self.0)
    }
}

impl Pid {
    pub fn interrupt(&self) -> anyhow::Result<Status> {
        kill((*self).into(), Signal::SIGINT).context("Failed to inetrrupt the process.")?;
        Ok(Status::Signaled(Signal::SIGINT))
    }

    pub fn suspend(&self) -> anyhow::Result<Status> {
        kill((*self).into(), Signal::SIGSTOP).context("Failed to inetrrupt the process.")?;
        Ok(Status::Signaled(Signal::SIGSTOP))
    }

    pub fn restart(&self) -> anyhow::Result<Status> {
        kill((*self).into(), Signal::SIGCONT).context("Failed to inetrrupt the process.")?;
        Ok(Status::Signaled(Signal::SIGCONT))
    }


    pub fn wait(&self) -> anyhow::Result<Status> {
        let (code, status, is_error) = unsafe {
            let mut siginfo = std::mem::zeroed();
            let error = libc::waitid(
                libc::P_PID,
                self.0 as u32,
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Status {
    Exited(i32),
    Signaled(Signal),
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Status::Exited(c) => write!(f, "exited: {}", c),
            Status::Signaled(s) => write!(f, "signaled: {}", s),
        }
    }
}
