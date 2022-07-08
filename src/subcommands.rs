use crate::config::ConfigFile;
use crate::post::Post;
use crate::{home, Start};
use anyhow::{Context, Result};
use derive_more::{Display, Error};
use std::fs;
use std::fs::File;
use std::io::ErrorKind;

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

pub fn start(option: Start) -> Result<()> {
    let mut www = std::process::Command::new("selfblog-server");
    let mut gemini = std::process::Command::new("selfblog-gemini");
    match option {
        Start::Www => {
            log::info!("HTTP server initialization...");
            www.spawn()?;
        }
        Start::Gemini => {
            log::info!("Gemini server initialization...");
            gemini.spawn()?;
        }
        Start::All => {
            log::info!("Starting all servers...");
            www.spawn()?;
            gemini.spawn()?;
        }
    }

    Ok(())
}

pub fn stop() -> Result<()> {
    let files = ["/tmp/selfblog-gemini.pid", "/tmp/selfblog-www.pid"];
    for file in files {
        log::debug!("Reading \"{file}\"...");
        match fs::read_to_string(file) {
            Ok(f) => {
                log::info!("({file}) Stopping server...");
                std::process::Command::new("kill").arg(f).spawn()?;
                fs::remove_file(file)
                    .with_context(|| "Not found \"{file}\"")?;
            }
            Err(e) => match e.kind() {
                ErrorKind::NotFound => log::debug!("({file}) Not found running server! Ignore..."),
                _ => return Err(e)?,
            },
        };
    }

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
