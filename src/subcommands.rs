use crate::config::ConfigFile;
use crate::home;
use anyhow::{Context, Error, Result};
use chrono::Datelike;
use derive_more::{Display, Error};
use pulldown_cmark::{html, Parser};
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::{ErrorKind, Write};

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
        Err(e) => return match e.kind() {
            ErrorKind::NotFound => Err(e).with_context(|| "Not found running server!"),
            _ => Err(Error::from(e)),
        },
    };

    log::info!("Stoping server...");
    std::process::Command::new("kill").arg(pid).spawn()?;

    log::debug!("Removing \"/tmp/selfblog-daemon.pid\"...");
    fs::remove_file("/tmp/selfblog-daemon.pid")?;

    Ok(())
}

#[derive(Serialize, Deserialize)]
struct Post {
    title: String,
    description: String,
}

pub fn new_post(title: &str, description: &str) -> Result<()> {
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

    log::debug!("Creating a post...");
    let posts_md_path = ConfigFile::new()?.blog.posts_md_path;
    let count_posts = fs::read_dir(&posts_md_path)?.count() + 1;
    let post_path = posts_md_path.join(format!("post-{count_posts}.md"));
    File::create(&post_path)?;

    log::debug!("Creating a hard link...");
    fs::hard_link(&post_path, home()?.join(".selfblog/.last_post"))?;

    log::debug!("Creating a tmp file...");
    let mut file = File::create(home()?.join(".selfblog/.new_post.lock"))?;

    log::debug!("Writing info to tmp file...");
    let post = Post {
        title: title.to_string(),
        description: description.to_string(),
    };
    file.write_all(toml::to_string(&post)?.as_bytes())?;

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
    if let Err(e) = File::open(home()?.join(".selfblog/.new_post.lock")) {
        log::error!("Not found '.new_post.lock'!");
        log::info!("First, create your post before marking it as ready!");
        return Err(e)?;
    }

    log::debug!("Reading lock file...");
    let tmp = home()?.join(".selfblog/.ready.lock");

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

    log::debug!("Creating lock file...");
    File::create(&tmp)?;

    log::debug!("Reading '.new_post.lock'...");
    let lock_file: Post = toml::from_str(&fs::read_to_string(
        home()?.join(".selfblog/.new_post.lock"),
    )?)?;

    let date = chrono::Local::now();
    let main_title = format!(
        "<p>{}-{:>02}-{:>02}: {}</p>",
        date.year(), date.month(), date.day(),
        &lock_file.title
    );

    log::debug!("Reading markdown from file...");
    let markdown = fs::read_to_string(home()?.join(".selfblog/.last_post"))?;
    let parser = Parser::new_ext(&markdown, pulldown_cmark::Options::all());
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    log::debug!("Reading configuration file...");
    let template_path = ConfigFile::new()?.blog.template_path;

    log::debug!("Reading template file...");
    let template = fs::read_to_string(&template_path)?
        .replace("[selfblog_main_title]", &main_title)
        .replace("[selfblog_post]", &html_output);

    log::debug!("Creating '.post_ready' file...");
    File::create(home()?.join(".selfblog/.post_ready"))?.write_all(template.as_bytes())?;

    log::info!("Done!");
    Ok(())
}

pub fn publish() -> Result<()> {
    log::debug!("Checking if '.ready.lock' already exists...");
    if let Err(e) = File::open(home()?.join(".selfblog/.ready.lock")) {
        log::error!("Not found '.ready.lock'!");
        log::info!("First, mark post as ready before publishing it!");
        return Err(e)?;
    }

    log::debug!("Reading post info...");
    let post_info: Post = toml::from_str(&fs::read_to_string(
        home()?.join(".selfblog/.new_post.lock"),
    )?)?;

    let website_path = ConfigFile::new()?.server.website_path;

    let count_posts = fs::read_dir(website_path.join("posts"))?.count() + 1;

    let post_path = website_path.join(
        format!("posts/post-{count_posts}.html")
    );

    log::debug!("Copying '.post_ready' to post_path...");
    fs::copy(home()?.join(".selfblog/.post_ready"), &post_path)?;

    log::debug!("Editing index.html...");
    let date = chrono::Local::today();
    let index = fs::read_to_string(&website_path.join("index.html"))?.replace(
        "<!-- [new_post_redirect] -->",
        &*format!(
            "<!-- [new_post_redirect] -->\n\
                <a href=\"posts/post-{}.html\" class=\"post\">\n\
                <p class=\"text title_text\">{}-{:>02}-{:>02}: {}</p>\n\
                <p class=\"text title_text description_text\">Description: {}</p>\n\
                </a>",
            count_posts + 1,
            date.year(),
            date.month(),
            date.day(),
            &post_info.title,
            &post_info.description
        ),
    );
    let mut file = File::create(&website_path.join("index.html"))?;
    file.write_all(index.as_bytes())?;

    log::debug!("Editing post...");
    let post = fs::read_to_string(&post_path)?.replace(
        "[selfblog_main_title]",
        &format!(
            "<p>{}-{:>02}-{:>02}: {}</p>",
            date.year(), date.month(), date.day(),
            &post_info.title
        )
    );
    let mut file = File::create(&post_path)?;
    file.write_all(post.as_bytes())?;

    log::debug!("Cleaning...");
    let selfblog_folder = home()?.join(".selfblog/");
    let files = [".new_post.lock", ".post_ready", ".ready.lock", ".last_post"];

    for p in files {
        fs::remove_file(selfblog_folder.join(p))?
    }

    log::info!("Your post are published!");

    Ok(())
}
