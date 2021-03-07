extern crate anyhow;

mod eval;
mod job;
mod parse;
mod session;

fn inner_main() -> anyhow::Result<()> {
    let mut session = session::Session::new(session::PromptReader::new())?;
    session.all()
}

fn main() {
    inner_main().unwrap_or_else(|e| {
        eprintln!("{}", e);
    })
}
