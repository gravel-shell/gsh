mod vars;

use vars::Vars;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NameSpace {
    vars: Vars,
}

impl NameSpace {
    pub fn push_var<T: Into<String>, U: AsRef<str>>(&mut self, key: T, value: U) {
        self.vars.push(key, value);
    }

    pub fn push_gvar<T: AsRef<str>, U: AsRef<str>>(&mut self, key: T, value: U) {
        self.vars.gpush(key, value);
    }

    pub fn mark(&mut self) {
        self.vars.mark();
    }

    pub fn drop(&mut self) {
        self.vars.drop();
    }
}
