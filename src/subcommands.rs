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
use std::path::PathBuf;

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
    template_path: PathBuf,
    post_path: PathBuf,
    post_info_path: PathBuf,
    posts_md_path: PathBuf
}

#[derive(Serialize, Deserialize, Default)]
struct PostInfo {
    title: String,
    description: String,
}

impl Post {
    pub fn new(post_id: usize) -> Result<Self> {
        let config = ConfigFile::new()?;
        let posts_md_path = config.blog.posts_md_path.clone();
        let post_info_path = home()?.join(format!(".selfblog/.post-{post_id}"));

        Ok(Self {
            template_path: config.blog.template_path,
            post_path: posts_md_path.join(format!("post-{post_id}.md")),
            post_info_path,
            posts_md_path,
        })
    }

    pub fn read_info(&mut self) -> Result<PostInfo> {
        Ok(toml::from_str(&fs::read_to_string(&self.post_info_path)?)?)
    }

    fn count_posts() -> Result<usize> {
        Ok(fs::read_dir(ConfigFile::new()?.blog.posts_md_path)?.count() / 2)
    }

    pub fn create(&self, title: String, description: String) -> Result<()> {
        File::create(&self.post_path)?;
        fs::hard_link(&self.post_path, home()?.join(".selfblog/.last_post"))?;

        let mut post_info = File::create(&self.post_info_path)?;
        post_info.write_all(toml::to_string(&PostInfo { title, description })?.as_bytes())?;
        fs::hard_link(&self.post_info_path, home()?.join(".selfblog/.new_post.lock"))?;

        Ok(())
    }

    pub fn ready(&mut self) -> Result<()> {
        let final_output = self.connect_md_with_template()?;
        let mut post_ready = File::create(home()?.join(".selfblog/.post_ready"))?;
        post_ready.write_all(final_output.as_bytes())?;

        Ok(())
    }

    pub fn update(&mut self) -> Result<()> {
        let final_output = self.connect_md_with_template()?;
        let mut post_ready = File::create(home()?.join(".selfblog/.post_ready"))?;
        post_ready.write_all(final_output.as_bytes())?;

        let mut new_post = File::create(home()?.join(".selfblog/.new_post.lock"))?;
        new_post.write_all(toml::to_string(&self.read_info()?)?.as_bytes())?;

        Ok(())
    }

    fn connect_md_with_template(&mut self) -> Result<String> {
        let markdown = fs::read_to_string(&self.post_path)?;
        let html_output = Post::md_to_html(&markdown);
        let template = fs::read_to_string(&self.template_path)?;

        Ok(self.add_html_to_template(&template, &html_output)?)
    }

    fn add_html_to_template(&mut self, template: &str, html: &str) -> Result<String> {
        let date = chrono::Local::now();
        let main_title = format!(
            "<p>{}-{:>02}-{:>02}: {}</p>",
            date.year(), date.month(), date.day(),
            &self.read_info()?.title
        );

        Ok(template.replace("[selfblog_main_title]", &main_title)
            .replace("[selfblog_post]", &html))
    }

    fn md_to_html(markdown: &str) -> String {
        let parser = Parser::new_ext(markdown, pulldown_cmark::Options::all());
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);
        html_output
    }
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
            ErrorKind::NotFound => {
                log::debug!("Creating lock file...");
                File::create(&tmp)?;
            }
            _ => return Err(e).with_context(|| "Error opening file!"),
        },
    }

    let mut post = Post::new(Post::count_posts()?)?;
    post.ready()?;

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
    let post_info: PostInfo = toml::from_str(&fs::read_to_string(
        home()?.join(".selfblog/.new_post.lock"),
    )?)?;

    let config_file = ConfigFile::new()?;
    let website_path = config_file.server.website_path;

    let count_posts = Post::count_posts()? + 1;

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
                <a href=\"posts/post-{}.html\">\n\
                <p class=\"{}\">{}-{:>02}-{:>02}: {}</p>\n\
                <p class=\"{}\">Description: {}</p>\n\
                </a>",
            count_posts,
            config_file.classes.title_text_main,
            date.year(),
            date.month(),
            date.day(),
            &post_info.title,
            config_file.classes.description_text_main,
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

pub fn update(post_id: usize) -> Result<()> {
    let mut post = Post::new(post_id)?;
    post.update()?;

    Ok(())
}