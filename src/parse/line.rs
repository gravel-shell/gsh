use super::{Command, Flow};

use combine::{Parser, Stream, choice};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Line {
    Command(Command),
    Flow(Flow),
}

impl Line {
    pub fn parse<I: Stream<Token = char>>() -> impl Parser<I, Output = Self> {
        choice((
            Flow::parse().map(|flow| Self::Flow(flow)),
            Command::parse().map(|cmd| Self::Command(cmd)),
        ))
    }

}
