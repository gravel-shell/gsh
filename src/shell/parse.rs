extern crate combine;

use anyhow::Context;
use super::{Job, Cmd, CmdKind};
use combine::parser::char::spaces;
use combine::{many1, optional, satisfy, sep_by1};
use combine::{Parser, Stream};

pub fn parse_line<T: AsRef<str>>(input: T) -> anyhow::Result<Job> {
    let (res, _) = job()
        .parse(input.as_ref())
        .context("Failed to parse cmd.")?;
    Ok(res)
}

fn job<Input>() -> impl Parser<Input, Output = Job>
where
    Input: Stream<Token = char>,
{
    optional(cmd()).map(|cmd| Job(cmd))
}

fn cmd<Input>() -> impl Parser<Input, Output = Cmd>
where
    Input: Stream<Token = char>,
{
    sep_by1(many1(satisfy(|c: char| !c.is_whitespace())), spaces()).map(|s: Vec<String>| {
        let mut s = s.into_iter();
        Cmd {
            kind: CmdKind::new(s.next().unwrap()),
            args: s.collect(),
        }
    })
}
