#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Redirect {
    Stdin(String),
    Stdout(String),
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Output {
    pub stdin: Option<String>,
    pub stdout: Option<String>,
}

impl Output {
    pub fn from(reds: Vec<Redirect>) -> Self {
        let mut res = Self::default();
        reds.into_iter().fold(&mut res, |acc, red| {
            match red {
                Redirect::Stdin(s) => acc.stdin = Some(s),
                Redirect::Stdout(s) => acc.stdout = Some(s),
            }
            acc
        });
        res
    }
}
