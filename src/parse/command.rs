use super::{spaces, spaces_line, Redirect, SpecialStr};
use combine::{attempt, eof, optional, sep_end_by, token};
use combine::{Parser, Stream};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Command {
    pub name: SpecialStr,
    pub args: Vec<Arg>,
    pub pipe: Option<Box<Command>>,
    pub bg: bool,
}

impl Command {
    fn empty() -> Self {
        Self {
            name: SpecialStr::new(),
            args: Vec::new(),
            pipe: None,
            bg: false,
        }
    }

    pub fn parse<I: Stream<Token = char>>() -> impl Parser<I, Output = Self> {
        command()
    }

    fn parse_<I: Stream<Token = char>>() -> impl Parser<I, Output = Self> {
        spaces_line().with(
            eof().map(|_| Self::empty()).or((
                SpecialStr::parse().skip(spaces()),
                sep_end_by(Arg::parse(), spaces()),
                optional(token('|').with(Self::parse())),
                optional(spaces().with(token('&')).skip(spaces())),
            )
                .map(|(name, args, pipe, bg)| Self {
                    name,
                    args,
                    pipe: pipe.map(Box::new),
                    bg: bg.is_some(),
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
    ExpandArg(SpecialStr),
    Arg(SpecialStr),
    Redirect(Redirect),
}

impl Arg {
    pub fn parse<I: Stream<Token = char>>() -> impl Parser<I, Output = Self> {
        attempt(Redirect::parse().map(Self::Redirect))
            .or(token('!').with(SpecialStr::parse().map(Self::ExpandArg)))
            .or(SpecialStr::parse().map(Self::Arg))
    }
}
