use std::env;

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct Vars {
    keys: Vec<String>,
    offsets: Vec<usize>,
}

impl Vars {
    pub fn set_args<T, U, US>(&mut self, name: T, args: US)
    where
        T: AsRef<str>,
        U: AsRef<str>,
        US: IntoIterator<Item = U>,
    {
        let name = name.as_ref();
        let args = args.into_iter().collect::<Vec<_>>();
        self.push("#", args.len().to_string());
        self.push("0", name);
        for (i, arg) in args.iter().enumerate() {
            self.push((i + 1).to_string(), arg);
        }
        self.push(
            "@",
            args.iter()
                .map(|arg| arg.as_ref())
                .collect::<Vec<_>>()
                .join(" "),
        );
    }

    pub fn push<T: Into<String>, U: AsRef<str>>(&mut self, key: T, value: U) {
        let key = key.into();
        let value = value.as_ref();
        let exists = env::var(&key).is_ok();
        env::set_var(&key, value);
        if !exists {
            self.keys.push(key);
        }
    }

    pub fn gpush<T: AsRef<str>, U: AsRef<str>>(&mut self, key: T, value: U) {
        let key = key.as_ref();
        let value = value.as_ref();
        env::set_var(key, value);
    }

    pub fn mark(&mut self) {
        let offset = self.keys.len();
        self.offsets.push(offset);
    }

    pub fn drop(&mut self) {
        let offset = self.offsets.pop().unwrap_or(0);
        for key in self.keys.drain(offset..) {
            env::remove_var(key);
        }
    }
}
