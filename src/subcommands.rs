use std::fs;
use std::fs::File;
use std::io::ErrorKind;
use daemonize::Daemonize;
use anyhow::{Error, Result};

pub async fn start() -> Result<()> {
    log::info!("HTTP server initialization...");
    let stdout = File::create("/tmp/selfblog.out").unwrap();
    let stderr = File::create("/tmp/selfblog.err").unwrap();

    let daemonize = Daemonize::new()
        .pid_file("/tmp/selfblog-daemon.pid")
        .stdout(stdout)
        .stderr(stderr);

    log::debug!("Starting daemon...");
    match daemonize.start() {
        Ok(_) => {
            log::debug!("Starting HTTP server...");
            let _ = super::server::rocket().launch().await?;
        }
        Err(e) => log::error!("Error: {}", e),
    }
    Ok(())
}

pub fn stop() -> Result<()> {
    log::debug!("Reading \"/tmp/selfblog-daemon.pid\"...");
    let pid = match fs::read_to_string("/tmp/selfblog-daemon.pid") {
        Ok(f) => f,
        Err(e) => match e.kind() {
            ErrorKind::NotFound => {
                log::error!("Not found running server!");
                std::process::exit(1)
            },
            _ => return Err(Error::from(e))
        }
    };
    log::info!("Stoping server...");
    std::process::Command::new("kill").arg(pid).spawn()?;
    log::debug!("Removing \"/tmp/selfblog-daemon.pid\"...");
    fs::remove_file("/tmp/selfblog-daemon.pid")?;
    Ok(())
}