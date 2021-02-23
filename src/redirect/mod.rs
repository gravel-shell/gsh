mod in_;
mod out;
mod output;

pub use in_::RedIn;
pub use out::{RedOut, RedOutKind, RedOutMode};
pub use output::Output;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Redirect {
    pub kind: RedKind,
    pub file: RedFile,
}
