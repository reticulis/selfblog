use crate::models::DatabaseUser;
use anyhow::{anyhow, Context, Result};
use sled::{Db, Iter};

pub struct Database {
    db: Db,
}

impl Database {
    pub fn new() -> Result<Self> {
        Ok(Self {
            db: sled::open(
                dirs::home_dir()
                    .context("Failed to get $HOME path")?
                    .join(".selfblog"),
            )?,
        })
    }
    pub fn get_user(&self, user: &[u8]) -> Result<DatabaseUser> {
        let tree = self.db.open_tree("users")?;

        Ok(match tree.get(user)? {
            Some(u) => {
                bincode::serde::decode_from_slice::<DatabaseUser, _>(
                    &u,
                    bincode::config::standard(),
                )?
                .0
            }
            None => return Err(anyhow!("Not found user!")),
        })
    }

    pub fn add_user(&self, login: &[u8], password: &[u8]) -> Result<()> {
        let tree = self.db.open_tree("users")?;

        let database_user = DatabaseUser::generate(password)?;

        tree.insert(
            login,
            bincode::serde::encode_to_vec(database_user, bincode::config::standard())?,
        )?;

        Ok(())
    }

    pub fn contains_user(&self, login: &[u8]) -> Result<bool> {
        tracing::info!("Contains user: {}", String::from_utf8_lossy(login));
        let tree = self.db.open_tree("users")?;

        Ok(tree.contains_key(login)?)
    }

    pub fn iter(&self) -> Iter {
        self.db.iter()
    }
}
