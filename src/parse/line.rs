use super::{spaces_line, Command};

use combine::{choice, Parser, Stream};
use combine::{sep_end_by, token};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Line {
    Single(Command),
    Multi(Vec<Line>),
}

impl Line {
    pub fn parse<I: Stream<Token = char>>() -> impl Parser<I, Output = Self> {
        line()
    }

    fn parse_<I: Stream<Token = char>>() -> impl Parser<I, Output = Self> {
        choice((
            multi().map(|lines| Self::Multi(lines)),
            Command::parse().map(|cmd| Self::Single(cmd)),
        ))
    }
}

combine::parser! {
    fn line[I]()(I) -> Line
    where [I: Stream<Token = char>]
    {
        Line::parse_()
    }
}

fn multi<I: Stream<Token = char>>() -> impl Parser<I, Output = Vec<Line>> {
    token('{')
        .skip(spaces_line())
        .with(sep_end_by(
            Line::parse(),
            token('\n').or(token(';')).with(spaces_line()),
        ))
        .skip(token('}'))
}
