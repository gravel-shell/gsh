use super::command::Command;

use combine::{Parser, Stream};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Line {
    Command(Command),
}

impl Line {
    pub fn parse<I: Stream<Token = char>>() -> impl Parser<I, Output = Self> {
        Command::parse().map(|cmd| Self::Command(cmd))
    }

}
