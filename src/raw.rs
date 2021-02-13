//! Raw function bundings.

use anyhow::Context;
use nix::libc;
use nix::sys::signal::Signal;
use nix::sys::wait::WaitPidFlag;
use nix::unistd::Pid;
use std::convert::TryFrom;
use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
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

pub fn waitid(pid: Pid, flag: WaitPidFlag) -> anyhow::Result<Status> {
    let (code, status, is_error) = unsafe {
        let mut siginfo = std::mem::zeroed();
        let error = libc::waitid(libc::P_PID, pid.as_raw() as u32, &mut siginfo, flag.bits());
        let siginfo = siginfo as libc::siginfo_t;
        (
            siginfo.si_code as i32,
            siginfo.si_status() as i32,
            error == -1,
        )
    };

    if is_error {
        Err(nix::Error::Sys(nix::errno::Errno::last())).context("Failed to wait the process.")?;
    }

    Ok(match code {
        libc::CLD_EXITED => Status::Exited(status),
        _ => Status::Signaled(Signal::try_from(status).context("Unnexpected signal.")?),
    })
}
