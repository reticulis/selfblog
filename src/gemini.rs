mod config;

use crate::config::ConfigFile;
use anyhow::{Context, Result};
use daemonize::Daemonize;
use futures_core::future::BoxFuture;
use futures_util::FutureExt;
use lazy_static::lazy_static;
use log::LevelFilter;
use std::fs::File;
use std::net::Ipv4Addr;
use twinstar::{Request, Response, ResponseHeader, Server};

lazy_static! {
    static ref CONFIG: ConfigFile = ConfigFile::new().unwrap();
}

fn main() -> Result<()> {
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .try_init()?;

    log::debug!("Creaing \"/tmp/selfblog-gemini.out\"...");
    let stdout = File::create("/tmp/selfblog-gemini.out")
        .with_context(|| "Failed creating \"/tmp/selfblog-gemini.out\"")?;

    log::debug!("Creaing \"/tmp/selfblog-gemini.err\"...");
    let stderr = File::create("/tmp/selfblog-gemini.err")
        .with_context(|| "Failed creating \"/tmp/selfblog-gemini.err\"")?;

    let daemonize = Daemonize::new()
        .pid_file("/tmp/selfblog-gemini.pid")
        .stdout(stdout)
        .stderr(stderr);

    log::debug!("Starting daemon...");
    match daemonize.start() {
        Ok(_) => {
            old_tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
                    let gemini = ConfigFile::new()?.gemini;
                    Server::bind((Ipv4Addr::from(gemini.address), gemini.port))
                        .add_route("/", handle_request)
                        .set_tls_dir(&CONFIG.gemini.certs_path)
                        .serve()
                        .await
                })?;
        }
        Err(e) => log::error!("Error: {}", e),
    };

    Ok(())
}

fn handle_request(request: Request) -> BoxFuture<'static, Result<Response>> {
    async move {
        if request.uri().path() == "/" {
            let path = &CONFIG.gemini.posts_path.join("index.gmi");
            let mime = twinstar::util::guess_mime_from_path(path);
            let response = twinstar::util::serve_file(path, &mime).await?;
            return Ok(response);
        }
        let path = request.path_segments();
        let response = twinstar::util::serve_dir(&CONFIG.gemini.posts_path, &path).await?;

        Ok(response)
    }
    .boxed()
}
