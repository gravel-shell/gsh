use crate::job::Jobs;
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

    pub fn exec(&self, jobs: &mut Jobs) -> anyhow::Result<()> {
        match self.kind {
            BuiltinKind::Empty => (),
            BuiltinKind::Exit => exit(&self.args)?,
            BuiltinKind::Cd => cd(&self.args)?,
            BuiltinKind::Fg => fg(&self.args, jobs)?,
            BuiltinKind::Jobs => println!("{:#?}", jobs),
            BuiltinKind::Var => var(&self.args)?,
            BuiltinKind::GVar => gvar(&self.args)?,
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
    Var,
    GVar,
}

impl BuiltinKind {
    pub fn new<T: AsRef<str>>(name: T) -> Option<Self> {
        Some(match name.as_ref() {
            "" => Self::Empty,
            "exit" => Self::Exit,
            "cd" => Self::Cd,
            "fg" => Self::Fg,
            "jobs" => Self::Jobs,
            "var" => Self::Var,
            "gvar" => Self::GVar,
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

pub fn var<T: AsRef<str>, TS: AsRef<[T]>>(args: TS) -> anyhow::Result<()> {
    use std::env;
    let args = args.as_ref();
    match args.len() {
        0 => {
            for (k, v) in env::vars() {
                println!("{}={:?}", k, v);
            }
        },
        1 => {
            let key = args[0].as_ref();
            let val = env::var(key)?;
            println!("{}={:?}", key, val);
        }
        2 => {
            let key = args[0].as_ref();
            let val = args[1].as_ref();
            env::set_var(key, val);
        }
        _ => anyhow::bail!("Unnexpected args number.")
    }

    Ok(())
}

pub fn gvar<T: AsRef<str>, TS: AsRef<[T]>>(args: TS) -> anyhow::Result<()> {
    use std::env;
    let args = args.as_ref();
    match args.len() {
        0 => {
            for (k, v) in env::vars() {
                println!("{}={:?}", k, v);
            }
        },
        1 => {
            let key = args[0].as_ref();
            let val = env::var(key)?;
            println!("{}={:?}", key, val);
        }
        2 => {
            let key = args[0].as_ref();
            let val = args[1].as_ref();
            env::set_var(key, val);
        }
        _ => anyhow::bail!("Unnexpected args number.")
    }

    Ok(())
}
