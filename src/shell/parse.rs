extern crate combine;
extern crate either;

use super::redirect::{RedFile, RedKind, RedOutMode, Redirect};
use super::{Cmd, CmdKind};
use anyhow::Context;
use combine::parser::char::spaces;
use combine::{attempt, eof, many1, one_of, satisfy, sep_end_by, token, value, choice, optional};
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

fn arg_or_red<I: Stream<Token = char>>() -> impl Parser<I, Output = Either<String, Redirect>> {
    attempt(redirect().map(|r| Either::Right(r))).or(string().map(|s| Either::Left(s)))
}

fn redirect<I: Stream<Token = char>>() -> impl Parser<I, Output = Redirect> {
    red_kind()
        .skip(spaces())
        .and(red_file())
        .map(|(kind, file)| Redirect { kind, file })
}

fn red_file<I: Stream<Token = char>>() -> impl Parser<I, Output = RedFile> {
    token('&')
        .with(one_of("012!".chars()).map(|c| match c {
            '0' => RedFile::Stdin,
            '1' => RedFile::Stdout,
            '2' => RedFile::Stderr,
            '!' => RedFile::Null,
            _ => unreachable!(),
        }))
        .or(string().map(|s| RedFile::File(s)))
}

fn red_kind<I: Stream<Token = char>>() -> impl Parser<I, Output = RedKind> {
    choice((
        token('1').and(token('>')).with(token('>')
            .map(|_| RedKind::Stdout(RedOutMode::Append))
            .or(value(RedKind::Stdout(RedOutMode::Overwrite)))),
        token('2').and(token('>')).with(token('>')
            .map(|_| RedKind::Stderr(RedOutMode::Append))
            .or(value(RedKind::Stderr(RedOutMode::Overwrite)))),
        token('<').map(|_| RedKind::Stdin),
        token('>').with(optional(token('>'))).map(|s| RedKind::Stdout(if s.is_some() {
            RedOutMode::Append
        } else {
            RedOutMode::Overwrite
        }))
    ))
}

fn string<I: Stream<Token = char>>() -> impl Parser<I, Output = String> {
    many1(satisfy(|c: char| !c.is_whitespace()))
}
