use crate::job::Jobs;
use crate::session::Vars;
use anyhow::Context;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Builtin {
    kind: BuiltinKind,
    args: Vec<String>,
}

impl Builtin {
    pub fn new<T, TS>(kind: BuiltinKind, args: TS) -> Self
    where
        T: Into<String>,
        TS: IntoIterator<Item = T>,
    {
        Self {
            kind,
            args: args.into_iter().map(|s| s.into()).collect(),
        }
    }

    pub fn exec(&self, jobs: &mut Jobs, vars: &mut Vars) -> anyhow::Result<()> {
        match self.kind {
            BuiltinKind::Empty => (),
            BuiltinKind::Exit => exit(&self.args)?,
            BuiltinKind::Cd => cd(&self.args)?,
            BuiltinKind::Fg => fg(&self.args, jobs)?,
            BuiltinKind::Jobs => println!("{:#?}", jobs),
            BuiltinKind::Let => let_(&self.args, vars)?,
            BuiltinKind::Export => export(&self.args, vars)?,
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinKind {
    Empty,
    Exit,
    Cd,
    Fg,
    Jobs,
    Let,
    Export,
}

impl BuiltinKind {
    pub fn new<T: AsRef<str>>(name: T) -> Option<Self> {
        Some(match name.as_ref() {
            "" => Self::Empty,
            "exit" => Self::Exit,
            "cd" => Self::Cd,
            "fg" => Self::Fg,
            "jobs" => Self::Jobs,
            "let" => Self::Let,
            "export" => Self::Export,
            _ => return None,
        })
    }
}

pub fn exit<T: AsRef<str>, TS: AsRef<[T]>>(args: TS) -> anyhow::Result<()> {
    let args = args.as_ref();
    let code = match args.len() {
        0 => 0,
        1 => args[0].as_ref()
            .parse::<i32>()
            .context("Failed to parse a number.")?,
        _ => anyhow::bail!("Unnexpected args number."),
    };
    std::process::exit(code);
}

pub fn cd<T: AsRef<str>, TS: AsRef<[T]>>(args: TS) -> anyhow::Result<()> {
    let args = args.as_ref();
    let path = match args.len() {
        0 => std::env::var("HOME").context("Failed to get the home directory.")?,
        1 => String::from(args[0].as_ref()),
        _ => anyhow::bail!("Unexpected args number."),
    };

    std::env::set_current_dir(path).context("Failed to set current dir.")?;

    Ok(())
}

pub fn fg<T: AsRef<str>, TS: AsRef<[T]>>(args: TS, jobs: &mut Jobs) -> anyhow::Result<()> {
    let args = args.as_ref();
    if args.len() != 1 {
        anyhow::bail!("Unexpected args number.");
    }

    let id = args[0].as_ref();

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

pub fn let_<T: AsRef<str>, TS: AsRef<[T]>>(args: TS, vars: &mut Vars) -> anyhow::Result<()> {
    let args = args.as_ref();
    if args.len() != 3 {
        anyhow::bail!("Unnexpected args number.");
    }

    if args[1].as_ref() != "=" {
        anyhow::bail!("Missing \"=\".");
    }

    vars.push(args[0].as_ref(), args[2].as_ref());
    Ok(())
}

pub fn export<T: AsRef<str>, TS: AsRef<[T]>>(args: TS, vars: &mut Vars) -> anyhow::Result<()> {
    let args = args.as_ref();
    if args.len() != 3 {
        anyhow::bail!("Unnexpected args number.");
    }

    if args[1].as_ref() != "=" {
        anyhow::bail!("Missing \"=\".");
    }

    vars.gpush(args[0].as_ref(), args[2].as_ref());
    Ok(())
}
