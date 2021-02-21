mod file;
mod in_;
mod kind;
mod out;
mod output;

pub use file::RedFile;
pub use in_::RedIn;
pub use kind::RedKind;
pub use out::{RedOut, RedOutKind, RedOutMode};
pub use output::Output;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Redirect {
    pub kind: RedKind,
    pub file: RedFile,
}
