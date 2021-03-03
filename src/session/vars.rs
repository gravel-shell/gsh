use std::env;

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct Vars {
    keys: Vec<String>,
    offsets: Vec<usize>,
}

impl Vars {
    pub fn push<T: Into<String>, U: AsRef<str>>(&mut self, key: T, value: U) {
        let key = key.into();
        let value = value.as_ref();
        env::set_var(&key, value);
        self.keys.push(key);
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
