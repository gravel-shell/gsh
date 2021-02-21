use super::Signal;
use std::fmt;

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

impl Status {
    pub fn stopped(&self) -> bool {
        match self {
            Self::Signaled(Signal::SIGSTOP)
            | Self::Signaled(Signal::SIGTSTP)
            | Self::Signaled(Signal::SIGTTIN)
            | Self::Signaled(Signal::SIGTTOU) => true,
            _ => false,
        }
    }

    pub fn continued(&self) -> bool {
        match self {
            Self::Signaled(Signal::SIGCONT) => true,
            _ => false,
        }
    }
}
