extern crate anyhow;

mod exec;
mod job;
mod parse;
mod prompt;
mod session;

fn inner_main() -> anyhow::Result<()> {
    let mut session = session::Session::<prompt::PromptReader>::new()?;
    session.all()
}

fn main() {
    inner_main().unwrap_or_else(|e| {
        eprintln!("{}", e);
    })
}
