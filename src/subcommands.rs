use anyhow::{Context, Error, Result};
use std::fs;
use std::fs::File;
use std::io::ErrorKind;
use crate::config::ConfigFile;
use derive_more::{Error, Display};

#[derive(Default, Debug, Error, Display)]
struct NewPostError;

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
    let website_path = super::config::ConfigFile::new(path)?.server.website_path;
    fs::create_dir_all(website_path + "/posts")
        .with_context(|| "Failed creating blog root directory")?;

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

pub fn new_post(title: &str) -> Result<()> {
    log::info!("Creating a new draft post...");

    log::debug!("Trying open a tmp file...");
    let tmp =dirs::home_dir()
        .with_context(|| "Failed getting home dir path!")?
        .join(".selfblog/.new_post.lock");

    match File::open(&tmp) {
        Ok(_) => {
            log::error!("{:?} is exists!", &tmp);
            return Err(NewPostError).with_context(||
                "Lock file is exists!\n\
                You already have a new draft post!"
            )
        }
        Err(e) => match e.kind() {
            ErrorKind::NotFound => {},
            _ => return Err(e).with_context(|| "Error opening file!")
        }
    }

    log::debug!("Creating a tmp file...");
    File::create(
        dirs::home_dir().with_context(|| "Failed getting home dir path!")?
        .join(".selfblog/.new_post.lock")
    )?;

    log::debug!("Reading configuration file...");
    let string = fs::read_to_string(
        dirs::home_dir().with_context(|| "Failed getting home dir path!")?
            .join(".selfblog/config.toml")
    )?;
    let config: ConfigFile = toml::from_str(&string)?;

    log::debug!("Creating a post...");
    let post_path = format!(
        "{}/posts/{}.md",
        config.server.website_path,
        title.split_whitespace()
            .collect::<Vec<&str>>()
            .join("_")
    );

    log::debug!("Checking if post with same title already exists...");
    match File::open(&post_path) {
        Ok(_) => {
            log::error!("Post with same title already exists!");
            return Err(NewPostError).with_context(|| "Post with same title already exists!")
        }
        Err(e) => match e.kind() {
            ErrorKind::NotFound => {
                File::create(&post_path)?;
            },
            _ => return Err(e).with_context(|| "Error checking old posts!")
        }
    }

    println!(
        "Post created successfully! \n\
        Now, edit your a new post and mark as ready to publish! \n\
        File post location: \"{}\"",
        &post_path
    );

    Ok(())
}