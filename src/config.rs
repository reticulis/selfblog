use anyhow::{Context, Result};
use derive_more::{Display, Error};
use serde_derive::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Default, Debug, Deserialize)]
pub struct ConfigFile {
    pub blog: Blog,
    pub server: Server,
}

#[derive(Default, Debug, Deserialize)]
pub struct Blog {
    pub title: String,
    pub author: String,
    pub template_path: String
}

#[derive(Default, Debug, Deserialize)]
pub struct Server {
    pub address: [u8; 4],
    pub port: u32,
    pub website_path: String,
}

#[derive(Default, Debug, Display, Error)]
struct ConfigParseError;

impl ConfigFile {
    pub fn new(config: PathBuf) -> Result<Self> {
        let toml = fs::read_to_string(config)?;
        let value: ConfigFile = toml::from_str(&toml)
            .with_context(|| "Failed parsing file!")?;

        Ok(value)
    }
}
