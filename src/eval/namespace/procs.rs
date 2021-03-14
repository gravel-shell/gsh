use crate::eval::Block;
use std::collections::HashMap;

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct Procs(HashMap<String, Block>);

impl Procs {
    pub fn push<T: Into<String>>(&mut self, name: T, block: Block) {
        let name = name.into();
        self.0.insert(name, block);
    }

    pub fn get<T: AsRef<str>>(&self, name: T) -> Option<Block> {
        self.0.get(name.as_ref()).cloned()
    }
}
