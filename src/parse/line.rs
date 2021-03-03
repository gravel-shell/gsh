use super::Command;

use combine::{Parser, Stream, choice};
use combine::{sep_by, skip_many1, token};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Line {
    Single(Command),
    Multi(Vec<Command>),
}

impl Line {
    pub fn parse<I: Stream<Token = char>>() -> impl Parser<I, Output = Self> {
        choice((
            multi().map(|cmds| Self::Multi(cmds)),
            Command::parse().map(|cmd| Self::Single(cmd)),
        ))
    }

}

fn multi<I: Stream<Token = char>>() -> impl Parser<I, Output = Vec<Command>> {
        token('{')
            .with(sep_by(
                Command::parse(),
                skip_many1(token('\n').or(token(';'))),
            ))
            .skip(token('}'))
}
