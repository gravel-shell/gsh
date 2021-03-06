use super::{spaces_line, Command, SpecialStr};

use combine::parser::char;
use combine::{attempt, choice, many, many1, optional, satisfy, sep_by, Parser, Stream};
use combine::{sep_end_by, token};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Block {
    Single(Command),
    Multi(Vec<Self>),
    If(SpecialStr, Box<Self>, Option<Box<Self>>),
    Case(SpecialStr, Vec<(Vec<SpecialStr>, Self)>),
    For(String, SpecialStr, Box<Self>),
    While(SpecialStr, Box<Self>),
    Proc(String, Box<Self>),
    Break,
    Continue,
}

impl Block {
    pub fn parse<I: Stream<Token = char>>() -> impl Parser<I, Output = Self> {
        block()
    }

    fn parse_<I: Stream<Token = char>>() -> impl Parser<I, Output = Self> {
        spaces_line().with(choice((
            attempt(char::string("break")).map(|_| Self::Break),
            attempt(char::string("continue")).map(|_| Self::Continue),
            proc().map(|(name, block)| Self::Proc(name, block)),
            while_().map(|(cond, block)| Self::While(cond, block)),
            for_().map(|(c, iter, block)| Self::For(c, iter, block)),
            case().map(|(cond, blocks)| Self::Case(cond, blocks)),
            if_().map(|(cond, first, second)| Self::If(cond, first, second)),
            multi().map(Self::Multi),
            Command::parse().map(Self::Single),
        )))
    }
}

combine::parser! {
    fn block[I]()(I) -> Block
    where [I: Stream<Token = char>]
    {
        Block::parse_()
    }
}

fn multi<I: Stream<Token = char>>() -> impl Parser<I, Output = Vec<Block>> {
    token('{')
        .skip(spaces_line())
        .with(sep_end_by(
            Block::parse(),
            token('\n').or(token(';')).with(spaces_line()),
        ))
        .skip(token('}'))
}

fn if_<I: Stream<Token = char>>(
) -> impl Parser<I, Output = (SpecialStr, Box<Block>, Option<Box<Block>>)> {
    (
        attempt(char::string("if")),
        spaces_line(),
        SpecialStr::parse(),
        spaces_line(),
        Block::parse().map(Box::new),
        spaces_line(),
        optional(
            (
                char::string("else"),
                spaces_line(),
                Block::parse().map(Box::new),
            )
                .map(|(_, _, line)| line),
        ),
    )
        .map(|(_, _, cond, _, first, _, second)| (cond, first, second))
}

fn case<I: Stream<Token = char>>(
) -> impl Parser<I, Output = (SpecialStr, Vec<(Vec<SpecialStr>, Block)>)> {
    (
        attempt(char::string("case")),
        spaces_line(),
        SpecialStr::parse(),
        spaces_line(),
        token('{'),
        spaces_line(),
        many(
            (
                sep_by(
                    SpecialStr::parse().skip(spaces_line()),
                    token('|').skip(spaces_line()),
                ),
                char::string("=>"),
                spaces_line(),
                Block::parse(),
                spaces_line(),
            )
                .map(|(pats, _, _, block, _)| (pats, block)),
        ),
        token('}'),
    )
        .map(|(_, _, cond, _, _, _, blocks, _)| (cond, blocks))
}

fn for_<I: Stream<Token = char>>() -> impl Parser<I, Output = (String, SpecialStr, Box<Block>)> {
    (
        attempt(char::string("for")),
        spaces_line(),
        many1(satisfy(|c: char| !c.is_whitespace())),
        spaces_line(),
        char::string("in"),
        spaces_line(),
        SpecialStr::parse(),
        spaces_line(),
        Block::parse().map(Box::new),
    )
        .map(|(_, _, c, _, _, _, iter, _, block)| (c, iter, block))
}

fn while_<I: Stream<Token = char>>() -> impl Parser<I, Output = (SpecialStr, Box<Block>)> {
    (
        attempt(char::string("while")),
        spaces_line(),
        SpecialStr::parse(),
        spaces_line(),
        Block::parse().map(Box::new),
    )
        .map(|(_, _, cond, _, block)| (cond, block))
}

fn proc<I: Stream<Token = char>>() -> impl Parser<I, Output = (String, Box<Block>)> {
    attempt((
        many1(satisfy(|c: char| !c.is_whitespace() && c != '{')),
        spaces_line(),
        combine::look_ahead(token('{')),
    ))
    .map(|(name, _, _)| name)
    .and(multi().map(|blocks| Box::new(Block::Multi(blocks))))
}
