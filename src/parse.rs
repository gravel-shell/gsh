extern crate combine;
use combine::parser::char::spaces;
use combine::{many1, optional, satisfy, sep_by1};
use combine::{ParseError, Parser, Stream};

#[derive(Debug)]
pub struct AstCmd {
    pub name: String,
    pub args: Vec<String>,
}

impl AstCmd {
    pub fn parse<T: AsRef<str>>(input: T) -> anyhow::Result<Option<Self>> {
        let (res, _rem) = optional(cmd()).parse(input.as_ref())?;

        Ok(res)
    }
}

fn cmd<Input>() -> impl Parser<Input, Output = AstCmd>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    sep_by1(many1(satisfy(|c: char| !c.is_whitespace())), spaces()).map(|s: Vec<String>| AstCmd {
        name: s[0].clone(),
        args: s[1..].into(),
    })
}
