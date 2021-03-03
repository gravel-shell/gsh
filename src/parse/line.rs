use super::{spaces_line, Command};

use combine::{choice, Parser, Stream};
use combine::{sep_end_by, token};

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
        .with(sep_end_by(
            Command::parse(),
            token('\n').or(token(';')).with(spaces_line()),
        ))
        .skip(token('}'))
}
