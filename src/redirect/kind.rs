use super::RedOutMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedKind {
    Stdin,
    HereDoc,
    Stdout(RedOutMode),
    Stderr(RedOutMode),
    Bind(RedOutMode),
}
