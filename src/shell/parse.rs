extern crate combine;
extern crate either;

use super::redirect::{RedFile, RedKind, RedOutMode, Redirect};
use super::{Cmd, CmdKind};
use combine::parser::char;
use combine::parser::repeat::skip_until;
use combine::{
    attempt, choice, count_min_max, eof, many, many1, one_of, optional, satisfy, sep_end_by, token,
    value,
};
use combine::{EasyParser, ParseError, Parser, Stream};
use either::Either;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Parsed {
    Complete(Cmd),
    Yet,
}

pub fn parse_line(input: &str) -> anyhow::Result<Parsed> {
    Ok(match cmd().easy_parse(input) {
        Ok((res, rem)) if rem.len() == 0 => Parsed::Complete(res),
        Ok(_) => anyhow::bail!("Unread characters are remain."),
        Err(e) if e.is_unexpected_end_of_input() => Parsed::Yet,
        Err(e) => anyhow::bail!(e.to_string()),
    })
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
        one_of("1-o".chars()).and(token('>')).with(
            token('>')
                .map(|_| RedKind::Stdout(RedOutMode::Append))
                .or(value(RedKind::Stdout(RedOutMode::Overwrite))),
        ),
        one_of("2=e".chars()).and(token('>')).with(
            token('>')
                .map(|_| RedKind::Stderr(RedOutMode::Append))
                .or(value(RedKind::Stderr(RedOutMode::Overwrite))),
        ),
        token('&').and(token('>')).with(
            token('>')
                .map(|_| RedKind::Bind(RedOutMode::Append))
                .or(value(RedKind::Bind(RedOutMode::Overwrite))),
        ),
        token('<').map(|_| RedKind::Stdin),
        token('>').with(optional(token('>'))).map(|s| {
            RedKind::Stdout(if s.is_some() {
                RedOutMode::Append
            } else {
                RedOutMode::Overwrite
            })
        }),
    ))
}

fn spaces<I: Stream<Token = char>>() -> impl Parser<I, Output = ()> {
    token('#')
        .and(skip_until(token('\n')))
        .map(|_| ())
        .or(char::spaces())
}

fn string<I: Stream<Token = char>>() -> impl Parser<I, Output = String> {
    choice((
        raw_str(),
        lit_str(),
        many1(satisfy(|c: char| !c.is_whitespace() && c != '#')),
    ))
}

fn lit_str<I: Stream<Token = char>>() -> impl Parser<I, Output = String> {
    use std::convert::TryFrom;

    token('"')
        .with(many(satisfy(|c| c != '"').then(|c| {
            if c == '\\' {
                choice((
                    one_of("abefnrtv\\\"".chars()).map(|seq| match seq {
                        'a' => '\x07',
                        'b' => '\x08',
                        'e' => '\x1b',
                        'f' => '\x0c',
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        'v' => '\x0b',
                        '\\' => '\\',
                        '"' => '"',
                        _ => unreachable!(),
                    }),
                    token('x')
                        .with(count_min_max(2, 2, char::hex_digit()))
                        .map(|s: String| u8::from_str_radix(s.as_str(), 16).unwrap() as char),
                    one_of("uU".chars())
                        .and(token('{'))
                        .with(many1(char::hex_digit()).map(|s: String| {
                            char::try_from(u32::from_str_radix(s.as_str(), 16).unwrap()).unwrap()
                        }))
                        .skip(token('}')),
                ))
                .left()
            } else {
                value(c).right()
            }
        })))
        .skip(token('"'))
}

fn raw_str<I: Stream<Token = char>>() -> impl Parser<I, Output = String> {
    token('\'')
        .with(many(choice((
            attempt(char::string("\\\\")).map(|_| '\\'),
            attempt(char::string("\\\'")).map(|_| '\''),
            satisfy(|c| c != '\''),
        ))))
        .skip(token('\''))
}
