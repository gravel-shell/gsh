extern crate unindent;

use super::Command;
use combine::parser::char;
use combine::{
    any, attempt, choice, count_min_max, many, many1, one_of, parser, satisfy, token, value,
};
use combine::{ParseError, Parser, Stream};
use unindent::unindent;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpecialStr(Vec<StrKind>);

#[derive(Clone, Debug, PartialEq, Eq)]
enum StrKind {
    String(String),
    Var(String),
    Cmd(Command),
    Pid(usize),
}

impl From<String> for SpecialStr {
    fn from(s: String) -> Self {
        Self(vec![StrKind::String(s)])
    }
}

impl SpecialStr {
    pub fn new() -> Self {
        SpecialStr(Vec::new())
    }

    pub fn parse<I: Stream<Token = char>>() -> impl Parser<I, Output = Self> {
        choice((
            attempt(raw_unindent()).map(Self::from),
            raw_str().map(|s| Self(vec![StrKind::String(s)])),
            attempt(lit_unindent()),
            lit(),
            direct(),
        ))
    }

    pub fn eval(&self, jobs: &crate::job::SharedJobs) -> anyhow::Result<String> {
        Ok(self
            .0
            .iter()
            .map(|kind| -> anyhow::Result<_> {
                match kind {
                    StrKind::String(s) => Ok(s.clone()),
                    StrKind::Var(key) => Ok(std::env::var(key)?),
                    StrKind::Cmd(cmd) => Ok(crate::eval::Command::from(cmd.clone())
                        .output(jobs)?
                        .trim()
                        .to_string()),
                    StrKind::Pid(id) => Ok(jobs.with(|jobs| jobs.get_pid(id))?.to_string()),
                }
            })
            .collect::<Result<Vec<_>, _>>()?
            .join(""))
    }
}

fn direct<I: Stream<Token = char>>() -> impl Parser<I, Output = SpecialStr> {
    many1(choice((
        command().map(StrKind::Cmd),
        env().map(StrKind::Var),
        pid().map(StrKind::Pid),
        direct_str().map(StrKind::String),
    )))
    .map(SpecialStr)
}

fn direct_str<I: Stream<Token = char>>() -> impl Parser<I, Output = String> {
    many1(satisfy(|c: char| {
        !c.is_whitespace() && "#|&;${}()".chars().all(|d| c != d)
    }))
}

fn lit_unindent<I: Stream<Token = char>>() -> impl Parser<I, Output = SpecialStr> {
    char::string("\"\"\"")
        .with(parser(|input: &mut I| {
            let (s, commited) = lit_str().parse_stream(input).into_result()?;
            let s = unindent(&s);
            let res = lit_reparse().parse_stream(&mut s.as_str()).into_result();

            match res {
                Ok((special, _)) => Ok((special, commited)),
                Err(_) => Err(combine::error::Commit::Peek(combine::error::Tracked::from(
                    I::Error::empty(input.position()),
                ))),
            }
        }))
        .skip(char::string("\"\"\""))
}

fn lit<I: Stream<Token = char>>() -> impl Parser<I, Output = SpecialStr> {
    token('"')
        .with(parser(|input: &mut I| {
            let (s, commited) = lit_str().parse_stream(input).into_result()?;
            let res = lit_reparse().parse_stream(&mut s.as_str()).into_result();

            match res {
                Ok((special, _)) => Ok((special, commited)),
                Err(_) => Err(combine::error::Commit::Peek(combine::error::Tracked::from(
                    I::Error::empty(input.position()),
                ))),
            }
        }))
        .skip(token('"'))
}

fn lit_str<I: Stream<Token = char>>() -> impl Parser<I, Output = String> {
    many(choice((
        token('\\').with(any()).map(|c| {
            if c == '"' {
                String::from(c)
            } else {
                format!("\\{}", c)
            }
        }),
        many1(satisfy(|c| c != '"' && c != '\\')),
    )))
    .map(|strs: Vec<_>| strs.join(""))
}

fn lit_reparse<I: Stream<Token = char>>() -> impl Parser<I, Output = SpecialStr> {
    use std::convert::TryFrom;

    many1(choice((
        command().map(StrKind::Cmd),
        env().map(StrKind::Var),
        pid().map(StrKind::Pid),
        many1(satisfy(|c| c != '$' && c != '(').then(|c| {
            if c == '\\' {
                choice((
                    one_of("abefnrtv%$(\\".chars()).map(|seq| match seq {
                        'a' => '\x07',
                        'b' => '\x08',
                        'e' => '\x1b',
                        'f' => '\x0c',
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        'v' => '\x0b',
                        '%' => '%',
                        '$' => '$',
                        '(' => '(',
                        '\\' => '\\',
                        _ => unimplemented!(),
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
        }))
        .map(StrKind::String),
    )))
    .map(SpecialStr)
}

fn raw_unindent<I: Stream<Token = char>>() -> impl Parser<I, Output = String> {
    char::string("''")
        .with(raw_str())
        .skip(char::string("''"))
        .map(|s| unindent(&s))
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

fn env<I: Stream<Token = char>>() -> impl Parser<I, Output = String> {
    token('$').with(
        token('{')
            .with(many1(satisfy(|c| c != '}')).skip(token('}')))
            .or(many1(satisfy(|c: char| {
                !c.is_whitespace() && c != '"' && c != '\'' && c != ')'
            }))),
    )
}

fn command<I: Stream<Token = char>>() -> impl Parser<I, Output = Command> {
    token('(').with(Command::parse()).skip(token(')'))
}

fn pid<I: Stream<Token = char>>() -> impl Parser<I, Output = usize> {
    token('%')
        .with(many1(char::digit()))
        .map(|id: String| id.parse().unwrap())
}
