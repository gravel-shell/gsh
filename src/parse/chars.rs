extern crate unindent;

use combine::parser::char;
use combine::parser::repeat::skip_until;
use combine::{
    attempt, choice, count_min_max, many, many1, one_of, satisfy, skip_many, token, value,
};
use combine::{Parser, Stream};
use unindent::unindent;

pub fn spaces<I: Stream<Token = char>>() -> impl Parser<I, Output = ()> {
    token('#')
        .and(skip_until(token('\n')))
        .map(|_| ())
        .or(skip_many(satisfy(|c: char| c.is_whitespace() && c != '\n')))
}

pub fn spaces_line<I: Stream<Token = char>>() -> impl Parser<I, Output = ()> {
    token('#')
        .and(skip_until(token('\n')))
        .map(|_| ())
        .or(char::spaces())
}

pub fn string<I: Stream<Token = char>>() -> impl Parser<I, Output = String> {
    choice((
        attempt(raw_unindent()),
        raw_str(),
        attempt(lit_unindent()),
        lit(),
        direct(),
    ))
}

fn direct<I: Stream<Token = char>>() -> impl Parser<I, Output = String> {
    many1(env().or(direct_str()))
}

fn direct_str<I: Stream<Token = char>>() -> impl Parser<I, Output = String> {
    many1(satisfy(|c: char| {
        !c.is_whitespace() && "#|&;${}()".chars().all(|d| c != d)
    }))
}

fn lit_unindent<I: Stream<Token = char>>() -> impl Parser<I, Output = String> {
    char::string("\"\"")
        .with(lit())
        .skip(char::string("\"\""))
        .map(|s| unindent(&s))
}

fn lit<I: Stream<Token = char>>() -> impl Parser<I, Output = String> {
    token('"').with(many(env().or(lit_str()))).skip(token('"'))
}

fn lit_str<I: Stream<Token = char>>() -> impl Parser<I, Output = String> {
    use std::convert::TryFrom;

    many1(satisfy(|c| c != '"' && c != '$').then(|c| {
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
        }))
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
    token('$')
        .with(many1(satisfy(|c| c != ';')))
        .skip(token(';'))
        .map(|s: String| std::env::var(s).unwrap())
}
