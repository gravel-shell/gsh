extern crate combine;

mod block;
mod chars;
mod command;
mod redirect;
mod string;

pub use block::Block;
pub use command::{Arg, Command};
pub use redirect::{RedKind, RedTarget, Redirect};
pub use string::SpecialStr;

use chars::{spaces, spaces_line};
use combine::{EasyParser, ParseError};
use combine::stream::position::Stream;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Parsed {
    Complete(Block),
    Yet,
}

pub fn parse_line(input: &str) -> anyhow::Result<Parsed> {
    Ok(match Block::parse().easy_parse(Stream::new(input)) {
        Ok((res, rem)) if rem.input.is_empty() => Parsed::Complete(res),
        Ok(_) => anyhow::bail!("Unread characters are remain."),
        Err(e) if e.is_unexpected_end_of_input() => Parsed::Yet,
        Err(e) => anyhow::bail!(e.to_string()),
    })
}
