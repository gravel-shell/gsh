use super::Reader;
use std::fs::File;
use std::io::{BufRead, BufReader, Lines};
use std::path::Path;

pub struct IOReader<R>(Lines<R>);

impl<R: BufRead> Reader for IOReader<R> {
    fn next_line(&mut self) -> anyhow::Result<Option<String>> {
        match self.0.next() {
            Some(line) => Ok(Some(line?)),
            None => Ok(None),
        }
    }
}

impl IOReader<BufReader<File>> {
    pub fn new_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Ok(Self(reader.lines()))
    }
}
