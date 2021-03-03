extern crate combine;

mod chars;
mod command;
mod line;
mod redirect;

pub use command::{Arg, Command};
pub use line::Line;
pub use redirect::{RedKind, RedTarget, Redirect};

use chars::{spaces, spaces_line, string};
use combine::{EasyParser, ParseError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Parsed {
    Complete(Line),
    Yet,
}

pub fn parse_line(input: &str) -> anyhow::Result<Parsed> {
    Ok(match Line::parse().easy_parse(input) {
        Ok((res, rem)) if rem.len() == 0 => Parsed::Complete(res),
        Ok(_) => anyhow::bail!("Unread characters are remain."),
        Err(e) if e.is_unexpected_end_of_input() => Parsed::Yet,
        Err(e) => anyhow::bail!(e.to_string()),
    })
}
