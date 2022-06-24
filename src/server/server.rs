use std::fs::File;
use rocket::fs::NamedFile;
use rocket::fs::{relative, FileServer};
use rocket::{routes, Build, Rocket};
use std::path::{Path, PathBuf};
use daemonize::Daemonize;
use anyhow::{Context, Result};

#[rocket::get("/<path..>")]
async fn blog(path: PathBuf) -> Option<NamedFile> {
    let mut path = Path::new(relative!("blog")).join(path);
    if path.is_dir() {
        path.push("index.html");
    }

    NamedFile::open(path).await.ok()
}

pub fn rocket() -> Rocket<Build> {
    rocket::build()
        .mount("/", routes![blog])
        .mount("/", FileServer::from(relative!("blog")))
}


#[tokio::main(flavor = "current_thread")]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let stdout = File::create("/tmp/selfblog.out")
        .with_context(|| "Failed creating \"/tmp/selfblog.out\"")?;
    let stderr = File::create("/tmp/selfblog.err")
        .with_context(|| "Failed creating \"/tmp/selfblog.out\"")?;

    let daemonize = Daemonize::new()
        .pid_file("/tmp/selfblog-daemon.pid")
        .stdout(stdout)
        .stderr(stderr);

    log::debug!("Starting daemon...");
    match daemonize.start() {
        Ok(_) => {
            log::debug!("Starting HTTP server...");
            let _ = rocket().launch().await?;
        }
        Err(e) => log::error!("Error: {}", e),
    };

    Ok(())
}