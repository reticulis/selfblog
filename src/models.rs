use crate::password::Password;
use anyhow::{anyhow, Result};
// use nanoid::nanoid;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct JsonUser {
    pub login: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct DatabaseUser {
    pub hash: String,
    // pub token: String,
}

impl DatabaseUser {
    pub fn generate(password: &[u8]) -> Result<Self> {
        let hash = match Password::new().hash_password(password) {
            Ok(h) => h,
            Err(_) => return Err(anyhow!("Failed hash password")),
        };

        Ok(Self {
            hash,
            // token: nanoid!(),
        })
    }
}
