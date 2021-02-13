extern crate anyhow;

mod jobs;
mod prompt;

fn main() {
    prompt::prompt().unwrap_or_else(|e| {
        eprintln!("{}", e);
    })
}
