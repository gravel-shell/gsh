use super::{Process, Status};
use crate::redirect::Output;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct SharedJobs(Arc<Mutex<Jobs>>);

impl SharedJobs {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(Jobs::new())))
    }

    pub fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }

    pub fn with<F, T>(&self, f: F) -> anyhow::Result<T>
    where
        F: FnOnce(&mut Jobs) -> anyhow::Result<T>
    {
        let mut lock = match self.0.lock() {
            Ok(l) => l,
            Err(e) => anyhow::bail!("Failed to get the lock: {}", e),
        };

        f(&mut lock)
    }

    pub fn get(&self) -> anyhow::Result<Jobs> {
        let lock = match self.0.lock() {
            Ok(l) => l,
            Err(e) => anyhow::bail!("Failed to get the lock: {}", e),
        };

        Ok((*lock).clone())
    }

    pub fn store(&self, jobs: Jobs) -> anyhow::Result<()> {
        let cloned = Arc::clone(&self.0);
        let mut lock = match cloned.lock() {
            Ok(l) => l,
            Err(e) => anyhow::bail!("Failed to get the lock: {}", e),
        };

        *lock = jobs;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Jobs(HashMap<usize, Process>);

impl Jobs {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn new_fg(&mut self, name: &str, args: Vec<String>, output: Output) -> anyhow::Result<()> {
        if self.0.contains_key(&0) {
            anyhow::bail!("The foreground process is already exist.");
        }

        self.0.insert(0, Process::new_cmd(name, args, output)?);
        Ok(())
    }

    pub fn wait_fg(&mut self) -> anyhow::Result<Option<Status>> {
        let res = match self.0.get(&0) {
            Some(proc) => Ok(Some(proc.wait()?)),
            None => Ok(None),
        };
        self.0.remove(&0);
        res
    }

    pub fn pop(&mut self, id: usize) -> Option<Process> {
        self.0.remove(&id)
    }
}
