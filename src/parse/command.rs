use super::{spaces, string, Redirect};
use combine::{attempt, eof, optional, sep_end_by, token};
use combine::{Parser, Stream};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Command {
    pub name: String,
    pub args: Vec<Arg>,
    pub pipe: Option<Box<Command>>,
}

impl Command {
    fn empty() -> Self {
        Self {
            name: String::new(),
            args: Vec::new(),
            pipe: None,
        }
    }

    pub fn parse<I: Stream<Token = char>>() -> impl Parser<I, Output = Self> {
        command()
    }

    fn parse_<I: Stream<Token = char>>() -> impl Parser<I, Output = Self> {
        spaces().with(
            eof().map(|_| Self::empty()).or((
                string().skip(spaces()),
                sep_end_by(Arg::parse(), spaces()),
                optional(token('|').with(Self::parse())),
            )
                .map(|(name, args, pipe)| Self {
                    name,
                    args,
                    pipe: pipe.map(|c| Box::new(c)),
                })),
        )
    }
}

combine::parser! {
    fn command[I]()(I) -> Command
    where [I: Stream<Token = char>]
    {
        Command::parse_()
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
