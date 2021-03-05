use combine::parser::char;
use combine::parser::repeat::skip_until;
use combine::{satisfy, skip_many, token};
use combine::{Parser, Stream};

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
