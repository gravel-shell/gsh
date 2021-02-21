#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RedFile {
    Stdin,
    Stdout,
    Stderr,
    Null,
    File(String),
}
