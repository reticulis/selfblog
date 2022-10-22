mod blog;
mod database;
mod services;

use crate::blog::Blog;
use crate::services::{default_handler, home_page, not_found, page_css};
use actix_files::{Files, NamedFile};
use actix_web::{web, App as ActixApp, HttpServer};
use anyhow::{anyhow, Result};
use clap::Parser;
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::fs::File;
use std::io::BufReader;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct App {
    /// Turn debugging information on
    #[clap(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[clap(short, long)]
    address: Option<IpAddr>,

    #[clap(short, long)]
    port: Option<u16>,

    #[clap(short, long)]
    key_path: Option<PathBuf>,

    #[clap(short, long)]
    cert_path: Option<PathBuf>,
}

impl App {
    async fn run(&self) -> Result<()> {
        let mut key = BufReader::new(File::open(
            self.key_path.as_ref().unwrap_or(&PathBuf::from("key.pem")),
        )?);
        let mut cert = BufReader::new(File::open(
            self.cert_path
                .as_ref()
                .unwrap_or(&PathBuf::from("cert.pem")),
        )?);

        let config = App::load_rustls(&mut key, &mut cert)?;

        let address = self
            .address
            .unwrap_or(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
        let port = self.port.unwrap_or(8443);

        let socket_addr = SocketAddr::new(address, port);

        let blog = Arc::new(Blog::new().unwrap());

        let blog = blog.clone();
        HttpServer::new(move || {
            ActixApp::new()
                .app_data(web::Data::new(blog.clone()))
                .service(web::resource("/").route(web::get().to(home_page)))
                .service(web::resource("/page/{number}").route(web::get().to(home_page)))
                .service(Files::new("/fonts", "html/fonts/"))
                .service(web::resource("/style.css").route(
                    web::get().to(|| async move { NamedFile::open("html/style.css").unwrap() }),
                ))
                .service(not_found)
                .default_service(web::to(default_handler))
        })
        .bind_rustls(socket_addr, config)?
        .run()
        .await?;

        Ok(())
    }

    fn load_rustls(key: &mut BufReader<File>, cert: &mut BufReader<File>) -> Result<ServerConfig> {
        let config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth();

        let cert_chain = certs(cert)?.into_iter().map(Certificate).collect();

        let mut keys: Vec<PrivateKey> = pkcs8_private_keys(key)?
            .into_iter()
            .map(PrivateKey)
            .collect();

        if keys.is_empty() {
            return Err(anyhow!("Could not locate PKCS 8 private keys."));
        }

        Ok(config.with_single_cert(cert_chain, keys.remove(0))?)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let app = App::parse();

    app.run().await?;

    Ok(())
}
