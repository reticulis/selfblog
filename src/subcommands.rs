use anyhow::{Context, Error, Result};
use std::fs;
use std::fs::File;
use std::io::{ErrorKind, Write};
use chrono::Datelike;
use crate::config::ConfigFile;
use derive_more::{Error, Display};
use pulldown_cmark::{html, Parser};

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
    let website_path = super::config::ConfigFile::new()?.server.website_path;
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

    log::debug!("Creating a post...");
    let post_path = dirs::home_dir()
        .with_context(|| "Failed getting home dir path!")?
        .join(".selfblog/post.md");
    File::create(&post_path)?;

    log::debug!("Creating a tmp file...");
    let mut file = File::create(
        dirs::home_dir().with_context(|| "Failed getting home dir path!")?
            .join(".selfblog/.new_post.lock")
    )?;

    log::debug!("Writing info to tmp file...");
    file.write(title.as_bytes())?;

    log::info!(
        "\nPost created successfully!\n\
        Now, edit your a new post and mark as ready to publish!\n\
        File post location: {:?}",
        &post_path
    );

    Ok(())
}

pub fn ready() -> Result<()> {
    log::debug!("Checking if '.new_post.lock' already exists...");
    if let Err(e) = File::open(
            dirs::home_dir()
            .with_context(|| "Failed getting home dir path!"
        )?.join(".selfblog/.new_post.lock")) {
        log::error!("Not found '.new_post.lock'!");
        log::info!("First, create your post before marking it as ready!");
        return Err(e)?
    }

    log::debug!("Reading lock file...");
    let tmp = dirs::home_dir()
        .with_context(|| "Failed getting home dir path!")?
        .join(".selfblog/.ready.lock");

    match File::open(&tmp) {
        Ok(_) => {
            log::error!("{:?} is exists!", &tmp);
            return Err(NewPostError).with_context(||
                "Lock file is exists!\n\
                You already have post ready!"
            )
        }
        Err(e) => match e.kind() {
            ErrorKind::NotFound => {},
            _ => return Err(e).with_context(|| "Error opening file!")
        }
    }

    log::debug!("Creating lock file...");
    File::create(&tmp)?;

    log::debug!("Reading '.new_post.lock'...");
    let lock_file = fs::read_to_string(
        dirs::home_dir()
            .with_context(|| "Failed getting home dir path!")?
            .join(".selfblog/.new_post.lock")
    )?;

    let date = chrono::Local::now();
    let main_title = format!(
        "<p>{}: {}</p>",
        format!("{}-{}-{}", date.year(), date.month(), date.day()),
        &lock_file
    );

    log::debug!("Reading markdown from file...");
    let markdown = fs::read_to_string(
        dirs::home_dir()
            .with_context(|| "Failed getting home dir path!")?
            .join(".selfblog/post.md")
    )?;
    let parser = Parser::new(&markdown);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    log::debug!("Reading configuration file...");
    let template_path = ConfigFile::new()?.blog.template_path;

    log::debug!("Reading template file...");
    let template = fs::read_to_string(&template_path)?
        .replace("[selfblog_main_title]", &main_title)
        .replace("[selfblog_post]", &html_output);

    log::debug!("Creating '.post_ready' file...");
    File::create(
        dirs::home_dir()
            .with_context(|| "Failed getting home dir path!")?
            .join(".selfblog/.post_ready")
    )?.write(template.as_bytes())?;

    log::info!("Done!");
    Ok(())
}