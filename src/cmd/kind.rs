use crate::job::Jobs;
use super::Redirects;
use anyhow::Context;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CmdKind {
    Empty,
    Exit,
    Cd,
    Fg,
    Jobs,
    Spawn,
    Cmd(String),
}

impl CmdKind {
    pub fn new<T: AsRef<str>>(name: T) -> Self {
        match name.as_ref() {
            "" => Self::Empty,
            "exit" => Self::Exit,
            "cd" => Self::Cd,
            "fg" => Self::Fg,
            "jobs" => Self::Jobs,
            "s" | "spawn" => Self::Spawn,
            s => Self::Cmd(s.into()),
        }
    }

    pub fn exec(self, jobs: &mut Jobs, args: Vec<String>, reds: Redirects) -> anyhow::Result<()> {
        match self {
            CmdKind::Empty => (),
            CmdKind::Exit => exit(args)?,
            CmdKind::Cd => cd(args)?,
            CmdKind::Fg => fg(args, jobs)?,
            CmdKind::Jobs => println!("{:#?}", jobs),
            CmdKind::Spawn => spawn(args, reds, jobs)?,
            CmdKind::Cmd(ref name) => jobs.new_fg(name, args, reds)?,
        }

        Ok(())
    }
}

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

pub fn fg(args: Vec<String>, jobs: &mut Jobs) -> anyhow::Result<()> {
    if args.len() != 1 {
        anyhow::bail!("Unexpected args number.");
    }

    let id = args.into_iter().next().unwrap();

    let id = if id.chars().next() == Some('%') {
        id.get(1..)
            .context("Unexpected end.")?
            .parse::<usize>()
            .context("Failed to parse a number.")?
    } else {
        jobs.from_pid(id.parse().context("Failed to parse a number.")?)
            .context("Can't find such a process.")?
    };

    jobs.to_fg(id)?;

    Ok(())
}

pub fn spawn(args: Vec<String>, reds: Redirects, jobs: &mut Jobs) -> anyhow::Result<()> {
    if args.len() == 0 {
        anyhow::bail!("Please specify the command to spawn.");
    }

    let mut args = args.into_iter();
    let name = args.next().unwrap();
    let args = args.collect();

    let (id, pid) = jobs.new_bg(&name, args, reds)?;

    println!("Spawned a new background process: %{} ({})", id, pid);

    Ok(())
}
