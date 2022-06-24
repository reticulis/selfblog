use anyhow::{Context, Error, Result};
use std::fs;
use std::io::ErrorKind;

pub fn init(config: &str) -> Result<()> {
    log::debug!("Creating \"selfblog\" directory...");
    let mut path = dirs::home_dir()
        .with_context(|| "Failed getting home dir path!")?
        .join(".selfblog/");

    match fs::create_dir(&path) {
        Ok(()) => (),
        Err(err) => match err.kind() {
            ErrorKind::AlreadyExists => {}
            _ => return Err(err).with_context(|| format!("Failed creating {:?}", &path))?,
        },
    }

    log::debug!("Copying configuration file to {:?}", &path);
    path.push(&config);
    fs::copy(&config, &path).with_context(|| "Failed copying configuration file!")?;

    log::debug!("Creating blog root directory...");
    let working_dir = super::config::Blog::new(&path)?.working_dir;
    fs::create_dir_all(working_dir + "/posts").with_context(|| "Failed creating blog root directory")?;

    log::info!("Done!");
    Ok(())
}

pub fn start() -> Result<()> {
    log::info!("HTTP server initialization...");
    std::process::Command::new("selfblog-server").spawn()?;
    Ok(())
}

pub fn stop() -> Result<()> {
    log::debug!("Reading \"/tmp/selfblog-daemon.pid\"...");
    let pid = match fs::read_to_string("/tmp/selfblog-daemon.pid") {
        Ok(f) => f,
        Err(e) => match e.kind() {
            ErrorKind::NotFound => return Err(e).with_context(|| "Not found running server!"),
            _ => return Err(Error::from(e)),
        },
    };

    log::info!("Stoping server...");
    std::process::Command::new("kill").arg(pid).spawn()?;

    log::debug!("Removing \"/tmp/selfblog-daemon.pid\"...");
    fs::remove_file("/tmp/selfblog-daemon.pid")?;

    Ok(())
}
