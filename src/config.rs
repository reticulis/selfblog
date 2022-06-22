use anyhow::Result;
use derive_more::{Display, Error};
use serde_derive::Deserialize;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

#[derive(Default, Debug, Deserialize)]
pub struct Blog {
    pub name: String,
    pub address: String,
    pub working_dir: String,
    pub template: String,
}

#[derive(Default, Debug, Display, Error)]
struct ConfigParseError;

#[allow(dead_code)]
impl Blog {
    pub fn new(config: &PathBuf) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let toml = fs::read_to_string(config)?;
        let value: Blog = toml::from_str(&toml)?;

        Ok(value)
    }
}
