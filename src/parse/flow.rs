use super::Command;

use combine::{sep_by, skip_many1, token};
use combine::{Parser, Stream};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Flow {
    Bracket(Vec<Command>),
}

impl Flow {
    pub fn parse<I: Stream<Token = char>>() -> impl Parser<I, Output = Self> {
        token('{')
            .with(sep_by(
                Command::parse(),
                skip_many1(token('\n').or(token(';'))),
            ))
            .skip(token('}'))
            .map(|cmds| Self::Bracket(cmds))
    }
}
