use super::{Process, Status};
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

    pub fn wait_fg(&self) -> anyhow::Result<Option<Status>> {
        let mut jobs = self.get()?;
        let status = jobs.wait_fg()?;
        self.store(jobs)?;
        Ok(status)
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

    pub fn new_fg(&mut self, pid: i32) -> anyhow::Result<()> {
        if self.0.contains_key(&0) {
            anyhow::bail!("The foreground process is already exist.");
        }

        self.0.insert(0, Process::from(pid));
        Ok(())
    }

    #[allow(dead_code)]
    pub fn new_bg(&mut self, pid: i32) -> anyhow::Result<(usize, i32)> {
        let id = self.get_available_id();
        let proc = Process::from(pid);
        self.0.insert(id, proc);
        Ok((id, pid))
    }

    pub fn wait_fg(&mut self) -> anyhow::Result<Option<Status>> {
        let res = match self.0.get(&0) {
            Some(proc) => Some(proc.wait()?),
            None => None,
        };
        let res = match res {
            Some(status) if status.stopped() => {
                let mut proc = self.0.remove(&0).unwrap();
                let id = self.get_available_id();
                eprintln!("\nSuspended: %{} ({})", id, proc.pid());
                proc.suspended = true;
                self.0.insert(id, proc);
                res
            }
            Some(status) if status.interrupted() => {
                anyhow::bail!("Interrupted");
            }
            Some(_) => {
                self.0.remove(&0);
                res
            }
            None => None,
        };
        Ok(res)
    }

    pub fn sigchld(&mut self) -> anyhow::Result<()> {
        let (pid, status) = match super::process::sigchld()? {
            Some(s) => s,
            None => return Ok(()),
        };

        if self.0.get(&0).map(|proc| proc.pid()) == Some(pid) {
            return Ok(());
        }

        let id = match self.from_pid(pid) {
            Some(id) => id,
            None => return Ok(()),
        };

        let mut proc = self.0.remove(&id).context("Failed to get the process.")?;

        match status {
            s if s.continued() => {
                eprintln!("\n[Background process %{} ({}) continued]", id, pid);
                proc.suspended = false;
                self.0.insert(id, proc);
            }
            s if s.stopped() => {
                eprintln!("\n[Background process %{} ({}) stopped]", id, pid);
                proc.suspended = true;
                self.0.insert(id, proc);
            }
            Status::Signaled(s) => {
                eprintln!(
                    "\n[Background process %{} ({}) terminated with signal \"{}\"]",
                    id, pid, s
                );
            }
            Status::Exited(c) => {
                eprintln!(
                    "\n[Background process %{} ({}) exited with code \"{}\"]",
                    id, pid, c
                );
            }
        }

        Ok(())
    }

    pub fn sigint(&mut self) -> anyhow::Result<()> {
        self.interrupt(0)?;
        Ok(())
    }

    pub fn sigtstp(&mut self) -> anyhow::Result<()> {
        self.suspend(0)
    }

    pub fn interrupt(&mut self, id: usize) -> anyhow::Result<Option<Status>> {
        let proc = self.0.remove(&id);
        if let Some(proc) = proc {
            proc.interrupt().map(Some)
        } else {
            Ok(None)
        }
    }

    pub fn suspend(&mut self, id: usize) -> anyhow::Result<()> {
        let proc = self.0.remove(&id);
        if let Some(mut proc) = proc {
            proc.suspend()?;
            self.0.insert(id, proc);
        }

        Ok(())
    }

    pub fn move_to_fg(&mut self, id: usize) -> anyhow::Result<()> {
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

    pub fn get_pid(&self, id: &usize) -> anyhow::Result<i32> {
        let proc = self
            .0
            .get(id)
            .context("Failed to get the process number.")?;
        Ok((*proc).into())
    }

    fn get_available_id(&self) -> usize {
        (1..).find(|i| !self.0.contains_key(&i)).unwrap()
    }
}
