use crate::config::ConfigFile;
use crate::home;
use anyhow::{Context, Error, Result};
use derive_more::{Display, Error};
use std::fs;
use std::fs::File;
use std::io::ErrorKind;
use crate::post::Post;

#[derive(Default, Debug, Error, Display)]
struct NewPostError;

pub fn init(config: &str) -> Result<()> {
    log::debug!("Creating \"selfblog\" directory...");
    let mut path = home()?.join(".selfblog/");

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
    let website_path = ConfigFile::new()?.server.website_path;
    fs::create_dir_all(website_path.join("posts"))
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
        Err(e) => {
            return match e.kind() {
                ErrorKind::NotFound => Err(e).with_context(|| "Not found running server!"),
                _ => Err(Error::from(e)),
            }
        }
    };

    log::info!("Stoping server...");
    std::process::Command::new("kill").arg(pid).spawn()?;

    log::debug!("Removing \"/tmp/selfblog-daemon.pid\"...");
    fs::remove_file("/tmp/selfblog-daemon.pid")?;

    Ok(())
}

pub fn new_post(title: String, description: String) -> Result<()> {
    log::info!("Creating a new draft post...");

    log::debug!("Trying open a tmp file...");
    let tmp = home()?.join(".selfblog/.new_post.lock");

    match File::open(&tmp) {
        Ok(_) => {
            log::error!("{:?} is exists!", &tmp);
            return Err(NewPostError).with_context(|| {
                "Lock file is exists!\n\
                You already have a new draft post!"
            });
        }
        Err(e) => match e.kind() {
            ErrorKind::NotFound => {}
            _ => return Err(e).with_context(|| "Error opening file!"),
        },
    }

    let count_posts = Post::count_posts()? + 1;
    let post = Post::new(count_posts)?;
    post.create(title, description)?;

    log::info!(
        "\nPost created successfully!\n\
        Now, edit your a new post and mark as ready to publish!\n\
        File post location: {:?}",
        &post.post_path
    );

    Ok(())
}

pub fn ready() -> Result<()> {
    log::debug!("Checking if '.new_post.lock' already exists...");
    if let Err(e) = File::open(home()?.join(".selfblog/.new_post.lock")) {
        log::error!("Not found '.new_post.lock'!");
        log::info!("First, create your post before marking it as ready!");
        return Err(e)?;
    }

    log::debug!("Reading lock file...");
    let tmp = home()?.join(".selfblog/.post_ready");

    match File::open(&tmp) {
        Ok(_) => {
            log::error!("{:?} is exists!", &tmp);
            return Err(NewPostError).with_context(|| {
                "Lock file is exists!\n\
                You already have post ready!"
            });
        }
        Err(e) => match e.kind() {
            ErrorKind::NotFound => {}
            _ => return Err(e).with_context(|| "Error opening file!"),
        },
    }

    let mut post = Post::new(Post::count_posts()?)?;
    post.ready()?;

    log::info!("Done!");
    Ok(())
}

pub fn publish() -> Result<()> {
    log::debug!("Checking if '.post_ready' already exists...");
    if let Err(e) = File::open(home()?.join(".selfblog/.post_ready")) {
        log::error!("Not found '.post_ready'!");
        log::info!("First, mark post as ready before publishing it!");
        return Err(e)?;
    }

    let post = Post::new(Post::count_posts()?)?;
    post.publish()?;

    log::debug!("Cleaning...");
    let selfblog_folder = home()?.join(".selfblog/");
    let files = [".new_post.lock", ".post_ready", ".last_post"];

    for p in files {
        fs::remove_file(selfblog_folder.join(p))?
    }

    log::info!("Your post are published!");

    Ok(())
}

pub fn update(post_id: usize) -> Result<()> {
    log::info!("Updating post...");
    let mut post = Post::new(post_id)?;
    post.update()?;

    log::info!("Done!");

    Ok(())
}

pub fn delete(post_id: usize) -> Result<()> {
    log::info!("Deleting post...");
    let post = Post::new(post_id)?;
    post.delete()?;

    log::info!("Done!");

    Ok(())
}