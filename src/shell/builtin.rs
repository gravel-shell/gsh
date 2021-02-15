use super::Output;
use crate::job::Pid;
use anyhow::Context;

pub fn exit(args: Vec<String>) -> anyhow::Result<()> {
    let code = match args.len() {
        0 => 0,
        1 => args[0]
            .parse::<i32>()
            .context("Failed to parse a number.")?,
        _ => anyhow::bail!("Unnexpected args number."),
    };
    std::process::exit(code);
}

pub fn cd(args: Vec<String>) -> anyhow::Result<()> {
    let path = match args.len() {
        0 => std::env::var("HOME").context("Failed to get the home directory.")?,
        1 => args.into_iter().next().unwrap(),
        _ => anyhow::bail!("Unexpected args number."),
    };

    std::env::set_current_dir(path).context("Failed to set current dir.")?;

    Ok(())
}

pub fn fg(args: Vec<String>) -> anyhow::Result<Pid> {
    if args.len() != 1 {
        anyhow::bail!("Unexpected args number.");
    }

    let id = args.into_iter().next().unwrap();

    let id = id.parse::<Pid>().context("Failed to parse a number.")?;
    id.restart()?;

    Ok(id)
}

pub fn cmd(name: &str, args: Vec<String>, output: Output) -> anyhow::Result<Pid> {
    use std::process::{Command, Stdio};
    let mut child = Command::new(name);
    child.args(args);

    if output.stdin != super::RedIn::Stdin {
        child.stdin(Stdio::piped());
    }

    if output.stdout != super::RedOut::stdout() {
        child.stdout(Stdio::piped());
    }

    if output.stderr != super::RedOut::stderr() {
        child.stderr(Stdio::piped());
    }

    let child = child
        .spawn()
        .context(format!("Invalid command: {}", name))?;

    let id = Pid::from(child.id() as i32);

    if output.stdin != super::RedIn::Stdin {
        std::io::copy(&mut output.stdin.to_reader()?, &mut child.stdin.unwrap())
            .context("Failed to redirect")?;
    }

    if output.stdout != super::RedOut::stdout() {
        std::io::copy(&mut child.stdout.unwrap(), &mut output.stdout.to_writer()?)
            .context("Failed to redirect")?;
    }

    if output.stderr != super::RedOut::stderr() {
        std::io::copy(&mut child.stderr.unwrap(), &mut output.stderr.to_writer()?)
            .context("Failed to redirect")?;
    }
    Ok(id)
}
