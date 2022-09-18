use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use actix_web::http::header::ContentType;
use anyhow::Result;
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};

#[get("/")]
async fn home_page() -> impl Responder {
    HttpResponse::Ok().content_type(ContentType::html()).body(
        "<h1>Hello world!</h1>"
    )
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = load_rustls()?;

    HttpServer::new(|| {
        App::new()
            .service(welcome)
    })
        .bind_rustls("127.0.0.1:8443", config)?
        .run()
        .await?;

    Ok(())
}

fn load_rustls() -> Result<ServerConfig> {
    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth();

    let mut cert = load_cert()?;

    let cert_chain = certs(&mut cert.0)?
        .into_iter()
        .map(Certificate)
        .collect();

    let mut keys: Vec<PrivateKey> = pkcs8_private_keys(&mut cert.1)?
        .into_iter()
        .map(PrivateKey)
        .collect();

    if keys.is_empty() {
        return Err(anyhow::Error::msg("Could not locate PKCS 8 private keys."))
    }

    Ok(config.with_single_cert(cert_chain, keys.remove(0))?)
}

fn load_cert() -> Result<(BufReader<File>, BufReader<File>)> {
    let cert_file = BufReader::new(File::open("cert.pem")?);
    let key_file = BufReader::new(File::open("key.pem")?);

    Ok((cert_file, key_file))
}
