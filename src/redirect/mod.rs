mod kind;
mod file;
mod output;
mod out;
mod in_;

pub use kind::RedKind;
pub use file::RedFile;
pub use output::Output;
pub use out::{RedOut, RedOutKind, RedOutMode};
pub use in_::RedIn;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Redirect {
    pub kind: RedKind,
    pub file: RedFile,
}

