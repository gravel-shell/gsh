use crate::redirect::Output;
use crate::job::Jobs;
use anyhow::Context;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CmdKind {
    Empty,
    Exit,
    Cd,
    Fg,
    Cmd(String),
}

impl CmdKind {
    pub fn new<T: AsRef<str>>(name: T) -> Self {
        match name.as_ref() {
            "exit" => Self::Exit,
            "cd" => Self::Cd,
            "fg" => Self::Fg,
            s => Self::Cmd(s.into()),
        }
    }

    pub fn exec(self, jobs: &mut Jobs, args: Vec<String>, output: Output) -> anyhow::Result<()> {
        match self {
            CmdKind::Empty => (),
            CmdKind::Exit => exit(args)?,
            CmdKind::Cd => cd(args)?,
            // CmdKind::Fg => fg(args, jobs)?,
            CmdKind::Fg => (),
            CmdKind::Cmd(ref name) => jobs.new_fg(name, args, output)?,
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

// pub fn fg(args: Vec<String>) -> anyhow::Result<()> {
//     if args.len() != 1 {
//         anyhow::bail!("Unexpected args number.");
//     }

//     let id = args.into_iter().next().unwrap();

//     let mut id = id.parse::<Process>().context("Failed to parse a number.")?;
//     id.restart()?;

//     Ok(id)
// }
