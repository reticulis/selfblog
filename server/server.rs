
use std::fs;
use std::fs::File;
use rocket::fs::FileServer;
use rocket::{Build, Rocket};
use daemonize::Daemonize;
use anyhow::{Context, Result};
use log::LevelFilter;
use derive_more::{Error, Display};
use toml::Value;

#[derive(Default, Debug, Display, Error)]
struct HomeDirParseError;

pub fn rocket(path: &str) -> Rocket<Build> {
    rocket::build()
        .mount("/", FileServer::from(path))
}

#[tokio::main(flavor = "current_thread")]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::builder().filter_level(LevelFilter::Debug).try_init()?;

    log::debug!("Creaing \"/tmp/selfblog.out\"...");
    let stdout = File::create("/tmp/selfblog.out")
        .with_context(|| "Failed creating \"/tmp/selfblog.out\"")?;

    log::debug!("Creaing \"/tmp/selfblog.err\"...");
    let stderr = File::create("/tmp/selfblog.err")
        .with_context(|| "Failed creating \"/tmp/selfblog.err\"")?;

    let daemonize = Daemonize::new()
        .pid_file("/tmp/selfblog-daemon.pid")
        .stdout(stdout)
        .stderr(stderr);

    log::debug!("Starting daemon...");
    let _ = match daemonize.start() {
        Ok(_) => {
            log::debug!("Reading configuration file...");
            let value = fs::read_to_string(
                dirs::home_dir()
                    .with_context(|| "Failed getting home dir path!")?
                    .join(".selfblog/config.toml")
            )?.parse::<Value>()?;

            log::debug!("Getting \"website_path\" value from configuration file...");
            let website_path = value
                .get("website_path")
                .with_context(|| "Failed getting website_path from config file!")?
                .as_str()
                .with_context(|| "Failed getting website_path from config file!")?;

            log::debug!("Launching HTTP server");
            let _ = rocket(website_path).launch().await?;
        }
        Err(e) => log::error!("Error: {}", e),
    };

    Ok(())
}