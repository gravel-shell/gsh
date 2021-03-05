use super::{spaces, SpecialStr};
use combine::{choice, one_of, token, value};
use combine::{Parser, Stream};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Redirect {
    pub kind: RedKind,
    pub target: RedTarget,
}

impl Redirect {
    pub fn parse<I: Stream<Token = char>>() -> impl Parser<I, Output = Self> {
        RedKind::parse()
            .skip(spaces())
            .and(RedTarget::parse())
            .map(|(kind, target)| Self { kind, target })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RedTarget {
    Stdin,
    Stdout,
    Stderr,
    Null,
    Other(SpecialStr),
}

impl RedTarget {
    pub fn parse<I: Stream<Token = char>>() -> impl Parser<I, Output = Self> {
        token('&')
            .with(one_of("012!".chars()))
            .map(|c| match c {
                '0' => Self::Stdin,
                '1' => Self::Stdout,
                '2' => Self::Stderr,
                '!' => Self::Null,
                _ => unreachable!(),
            })
            .or(SpecialStr::parse().map(|s| Self::Other(s)))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RedKind {
    AppendStdout,
    OverwriteStdout,
    AppendStderr,
    OverwriteStderr,
    AppendBoth,
    OverwriteBoth,
    Stdin,
    HereDoc,
}

impl RedKind {
    pub fn parse<I: Stream<Token = char>>() -> impl Parser<I, Output = Self> {
        choice((
            one_of("1-o".chars()).and(token('>')).with(
                token('>')
                    .map(|_| Self::AppendStdout)
                    .or(value(Self::OverwriteStdout)),
            ),
            one_of("2=e".chars()).and(token('>')).with(
                token('>')
                    .map(|_| Self::AppendStderr)
                    .or(value(Self::OverwriteStderr)),
            ),
            token('&').and(token('>')).with(
                token('>')
                    .map(|_| Self::AppendBoth)
                    .or(value(Self::OverwriteBoth)),
            ),
            token('<').with(
                one_of("<-=h".chars())
                    .map(|_| Self::HereDoc)
                    .or(value(Self::Stdin)),
            ),
            token('>').with(
                token('>')
                    .map(|_| Self::AppendStdout)
                    .or(value(Self::OverwriteStdout)),
            ),
        ))
    }
}
