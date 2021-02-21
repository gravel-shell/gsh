use super::{Process, Status};
use crate::redirect::Output;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use anyhow::Context;

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
        F: FnOnce(&mut Jobs) -> anyhow::Result<T>,
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
            Some(proc) => Some(proc.wait()?),
            None => None,
        };
        let res = match res {
            // Don't delete process if signaled "Stop".
            Some(status) if status.stopped() => None,
            Some(_) => {
                self.0.remove(&0);
                res
            }
            None => None,
        };
        Ok(res)
    }

    pub fn interrupt(&mut self, id: usize) -> anyhow::Result<Option<Status>> {
        let proc = self.0.remove(&id);
        if let Some(proc) = proc {
            proc.interrupt().map(|s| Some(s))
        } else {
            Ok(None)
        }
    }

    pub fn suspend(&mut self, id: usize) -> anyhow::Result<Option<(usize, i32)>> {
        let proc = self.0.remove(&id);
        if let Some(mut proc) = proc {
            proc.suspend()?;
            let id = if id == 0 { self.get_available_id() } else { id };
            self.0.insert(id, proc);
            Ok(Some((id, proc.pid())))
        } else {
            Ok(None)
        }
    }

    pub fn to_fg(&mut self, id: usize) -> anyhow::Result<()> {
        if id == 0 {
            return Ok(());
        }

        if self.0.contains_key(&0) {
            anyhow::bail!("The foreground process is already exist.");
        }

        let mut proc = self.0.remove(&id).context("Can't find such a process.")?;
        if proc.suspended() {
            proc.restart()?;
        }

        self.0.insert(0, proc);
        Ok(())
    }

    pub fn from_pid(&self, pid: i32) -> Option<usize> {
        self.0.iter().find(|(_, v)| v.pid() == pid).map(|(k, _)| *k)
    }

    fn get_available_id(&self) -> usize {
        (1..).find(|i| !self.0.contains_key(&i)).unwrap()
    }
}
