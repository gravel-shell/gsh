use super::{spaces, string, Redirect};
use combine::{attempt, eof, sep_end_by};
use combine::{Parser, Stream};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Command {
    pub name: String,
    pub args: Vec<Arg>,
}

impl Command {
    fn empty() -> Self {
        Self {
            name: String::new(),
            args: Vec::new(),
        }
    }

    pub fn parse<I: Stream<Token = char>>() -> impl Parser<I, Output = Self> {
        spaces().with(
            eof().map(|_| Self::empty()).or((
                string().skip(spaces()),
                sep_end_by(Arg::parse(), spaces()),
            )
                .map(|(name, args)| Self { name, args })),
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Arg {
    Arg(String),
    Redirect(Redirect),
}

impl Arg {
    pub fn parse<I: Stream<Token = char>>() -> impl Parser<I, Output = Self> {
        attempt(Redirect::parse().map(|r| Self::Redirect(r))).or(string().map(|s| Self::Arg(s)))
    }
}
