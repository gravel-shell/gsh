extern crate combine;

mod chars;
mod command;
mod redirect;

pub use command::{Arg, Command};
pub use redirect::{RedKind, RedTarget, Redirect};

use chars::{spaces, string};
use combine::{EasyParser, ParseError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Parsed {
    Complete(Command),
    Yet,
}

pub fn parse_line(input: &str) -> anyhow::Result<Parsed> {
    Ok(match Command::parse().easy_parse(input) {
        Ok((res, rem)) if rem.len() == 0 => Parsed::Complete(res),
        Ok(_) => anyhow::bail!("Unread characters are remain."),
        Err(e) if e.is_unexpected_end_of_input() => Parsed::Yet,
        Err(e) => anyhow::bail!(e.to_string()),
    })
}