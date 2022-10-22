use anyhow::{Context, Result};
use sled::{Db, Iter};

pub struct Database {
    db: Db,
}

impl Database {
    pub fn new() -> Result<Self> {
        let db = sled::open(
            dirs::home_dir()
                .context("Failed to get $HOME path")?
                .join(".selfblog"),
        )?;

        Ok(Self { db })
    }

    pub fn iter(&self) -> Iter {
        self.db.iter()
    }
}
