mod vars;
mod procs;

use vars::Vars;
use procs::Procs;

use crate::eval::Block;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NameSpace {
    vars: Vars,
    procs: Procs,
}

impl NameSpace {
    pub fn push_var<T: Into<String>, U: AsRef<str>>(&mut self, key: T, value: U) {
        self.vars.push(key, value);
    }

    pub fn push_gvar<T: AsRef<str>, U: AsRef<str>>(&mut self, key: T, value: U) {
        self.vars.gpush(key, value);
    }

    pub fn push_proc<T: Into<String>>(&mut self, name: T, block: Block) {
        self.procs.push(name, block);
    }

    pub fn get_proc<T: AsRef<str>>(&mut self, name: T) -> Option<&Block> {
        self.procs.get(name)
    }

    pub fn mark(&mut self) {
        self.vars.mark();
    }

    pub fn drop(&mut self) {
        self.vars.drop();
    }
}
