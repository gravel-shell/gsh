use super::{spaces_line, Command, SpecialStr};

use combine::parser::char;
use combine::{choice, optional, Parser, Stream};
use combine::{sep_end_by, token};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Line {
    Single(Command),
    Multi(Vec<Line>),
    If(SpecialStr, Box<Line>, Option<Box<Line>>),
    Break,
    Continue,
}

impl Line {
    pub fn parse<I: Stream<Token = char>>() -> impl Parser<I, Output = Self> {
        line()
    }

    fn parse_<I: Stream<Token = char>>() -> impl Parser<I, Output = Self> {
        choice((
            char::string("break").map(|_| Self::Break),
            char::string("continue").map(|_| Self::Continue),
            if_().map(|(cond, first, second)| Self::If(cond, first, second)),
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

fn if_<I: Stream<Token = char>>(
) -> impl Parser<I, Output = (SpecialStr, Box<Line>, Option<Box<Line>>)> {
    (
        char::string("if"),
        spaces_line(),
        SpecialStr::parse(),
        spaces_line(),
        Line::parse().map(|line| Box::new(line)),
        spaces_line(),
        optional(
            (
                char::string("else"),
                spaces_line(),
                Line::parse().map(|line| Box::new(line)),
            )
                .map(|(_, _, line)| line),
        ),
    )
        .map(|(_, _, cond, _, first, _, second)| (cond, first, second))
}
