use std::sync::{Arc, Mutex};

use crate::jobs::Pid;

#[derive(Debug)]
pub struct CurPid(Arc<Mutex<Option<Pid>>>);

impl CurPid {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(None)))
    }

    pub fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }

    pub fn get(&self) -> anyhow::Result<Option<Pid>> {
        let lock = match self.0.lock() {
            Ok(l) => l,
            Err(e) => anyhow::bail!("Failed to get the lock: {}", e),
        };

        Ok(*lock)
    }

    pub fn store(&self, pid: Pid) -> anyhow::Result<()> {
        let cloned = Arc::clone(&self.0);
        let mut lock = match cloned.lock() {
            Ok(l) => l,
            Err(e) => anyhow::bail!("Failed to get the lock: {}", e),
        };

        *lock = Some(pid);
        Ok(())
    }

    pub fn reset(&self) -> anyhow::Result<()> {
        let cloned = Arc::clone(&self.0);
        let mut lock = match cloned.lock() {
            Ok(l) => l,
            Err(e) => anyhow::bail!("Failed to get the lock: {}", e),
        };

        *lock = None;
        Ok(())
    }
}
