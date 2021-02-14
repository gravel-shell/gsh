extern crate combine;
extern crate either;

use super::{Cmd, CmdKind, Job, Redirect};
use anyhow::Context;
use combine::parser::char::spaces;
use combine::{attempt, many1, optional, satisfy, sep_end_by, token};
use combine::{Parser, Stream};
use either::Either;

pub fn parse_line<T: AsRef<str>>(input: T) -> anyhow::Result<Job> {
    let (res, _) = job()
        .parse(input.as_ref())
        .context("Failed to parse line.")?;
    Ok(res)
}

fn job<I: Stream<Token = char>>() -> impl Parser<I, Output = Job> {
    optional(cmd()).map(|cmd| Job { cmd })
}

fn cmd<I: Stream<Token = char>>() -> impl Parser<I, Output = Cmd> {
    spaces()
        .with((kind().skip(spaces()), sep_end_by(arg_or_red(), spaces())))
        .map(|(kind, args_reds): (_, Vec<_>)| {
            let (args, redirects): (Vec<_>, Vec<_>) =
                args_reds.into_iter().partition(|e| e.is_left());
            let args: Vec<_> = args.into_iter().map(|s| s.unwrap_left()).collect();
            let redirects: Vec<_> = redirects.into_iter().map(|s| s.unwrap_right()).collect();
            Cmd {
                kind,
                args,
                redirects,
            }
        })
}

fn kind<I: Stream<Token = char>>() -> impl Parser<I, Output = CmdKind> {
    string().map(|s| CmdKind::new(s))
}

fn redirect<I: Stream<Token = char>>() -> impl Parser<I, Output = Redirect> {
    token('>')
        .skip(spaces())
        .with(string())
        .map(|to| Redirect { to })
}

fn arg_or_red<I: Stream<Token = char>>() -> impl Parser<I, Output = Either<String, Redirect>> {
    attempt(redirect().map(|r| Either::Right(r))).or(string().map(|s| Either::Left(s)))
}

fn string<I: Stream<Token = char>>() -> impl Parser<I, Output = String> {
    many1(satisfy(|c: char| !c.is_whitespace()))
}
