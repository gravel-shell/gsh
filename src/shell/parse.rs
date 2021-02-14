extern crate combine;
extern crate either;

use super::{Cmd, CmdKind, Redirect};
use anyhow::Context;
use combine::parser::char::spaces;
use combine::{attempt, eof, many1, one_of, satisfy, sep_end_by};
use combine::{Parser, Stream};
use either::Either;

pub fn parse_line<T: AsRef<str>>(input: T) -> anyhow::Result<Cmd> {
    let (res, _) = cmd()
        .parse(input.as_ref())
        .context("Failed to parse line.")?;
    Ok(res)
}

fn cmd<I: Stream<Token = char>>() -> impl Parser<I, Output = Cmd> {
    spaces().with(eof().map(|()| Cmd::empty()).or(
        (kind().skip(spaces()), sep_end_by(arg_or_red(), spaces())).map(
            |(kind, args_reds): (_, Vec<_>)| {
                let (args, redirects): (Vec<_>, Vec<_>) =
                    args_reds.into_iter().partition(|e| e.is_left());
                let args: Vec<_> = args.into_iter().map(|s| s.unwrap_left()).collect();
                let redirects: Vec<_> = redirects.into_iter().map(|s| s.unwrap_right()).collect();
                Cmd {
                    kind,
                    args,
                    redirects,
                }
            },
        ),
    ))
}

fn kind<I: Stream<Token = char>>() -> impl Parser<I, Output = CmdKind> {
    string().map(|s| CmdKind::new(s))
}

fn redirect<I: Stream<Token = char>>() -> impl Parser<I, Output = Redirect> {
    one_of("<>".chars())
        .skip(spaces())
        .and(string())
        .map(|(io, file)| match io {
            '<' => Redirect::Stdin(file),
            '>' => Redirect::Stdout(file),
            _ => unreachable!(),
        })
}

fn arg_or_red<I: Stream<Token = char>>() -> impl Parser<I, Output = Either<String, Redirect>> {
    attempt(redirect().map(|r| Either::Right(r))).or(string().map(|s| Either::Left(s)))
}

fn string<I: Stream<Token = char>>() -> impl Parser<I, Output = String> {
    many1(satisfy(|c: char| !c.is_whitespace()))
}
