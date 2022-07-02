use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use crate::config::ConfigFile;
use anyhow::Result;
use chrono::Datelike;
use pulldown_cmark::{html, Parser};
use crate::home;

#[derive(Serialize, Deserialize)]
pub struct Post {
    config: ConfigFile,
    post_id: usize,
    pub post_path: PathBuf,
    post_info_path: PathBuf,
}

#[derive(Serialize, Deserialize, Default)]
struct PostInfo {
    title: String,
    description: String,
}

impl Post {
    pub fn new(post_id: usize) -> Result<Self> {
        log::debug!("Reading config file...");
        let config = ConfigFile::new()?;
        let posts_md_path = config.blog.posts_md_path.clone();
        let post_info_path = posts_md_path.join(format!(".post-{post_id}"));

        Ok(Self {
            config,
            post_id,
            post_path: posts_md_path.join(format!("post-{post_id}.md")),
            post_info_path,
        })
    }

    fn read_info(&mut self) -> Result<PostInfo> {
        Ok(toml::from_str(&fs::read_to_string(&self.post_info_path)?)?)
    }

    pub fn count_posts() -> Result<usize> {
        Ok(fs::read_dir(ConfigFile::new()?.blog.posts_md_path)?.count() / 2)
    }

    pub fn create(&self, title: String, description: String) -> Result<()> {
        File::create(&self.post_path)?;
        fs::hard_link(&self.post_path, home()?.join(".selfblog/.last_post"))?;

        let mut post_info = File::create(&self.post_info_path)?;
        post_info.write_all(toml::to_string(&PostInfo { title, description })?.as_bytes())?;
        fs::hard_link(
            &self.post_info_path,
            home()?.join(".selfblog/.new_post.lock"),
        )?;

        Ok(())
    }

    pub fn ready(&mut self) -> Result<()> {
        let final_output = self.connect_md_with_template()?;
        log::debug!("Creating \".post_ready\" file...");
        let mut post_ready = File::create(home()?.join(".selfblog/.post_ready"))?;
        post_ready.write_all(final_output.as_bytes())?;

        Ok(())
    }

    pub fn update(&mut self) -> Result<()> {
        let final_output = self.connect_md_with_template()?;
        log::debug!("Creating \".post_ready\" file...");
        let mut post_ready = File::create(
            &self.config.server.website_path
                .join(format!("posts/post-{}.html", &self.post_id))
        )?;
        post_ready.write_all(final_output.as_bytes())?;

        Ok(())
    }

    pub fn publish(&self) -> Result<()> {
        log::debug!("Reading post info...");
        let post_info: PostInfo = toml::from_str(&fs::read_to_string(
            home()?.join(".selfblog/.new_post.lock"),
        )?)?;

        let website_path = &self.config.server.website_path;

        let post_path = website_path.join(format!("posts/post-{}.html", &self.post_id));

        log::debug!("Copying '.post_ready' to post_path...");
        fs::copy(home()?.join(".selfblog/.post_ready"), &post_path)?;

        log::debug!("Editing index.html...");
        let date = chrono::Local::today();

        let index = fs::read_to_string(&website_path.join("index.html"))?.replace(
            "<!-- [new_post_redirect] -->",
            &format!(
                "<!-- [new_post_redirect] -->\n\
                <a title=\"post-{}\" href=\"posts/post-{}.html\">\n\
                <p class=\"{}\">{}-{:>02}-{:>02}: {}</p>\n\
                <p class=\"{}\">Description: {}</p>\n\
                </a>",
                &self.post_id,
                &self.post_id,
                &self.config.classes.title_text_main,
                date.year(),
                date.month(),
                date.day(),
                &post_info.title,
                &self.config.classes.description_text_main,
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
                date.year(),
                date.month(),
                date.day(),
                &post_info.title
            ),
        );
        let mut file = File::create(&post_path)?;
        file.write_all(post.as_bytes())?;

        Ok(())
    }

    pub fn delete(&self) -> Result<()> {
        let website_path = &self.config.server.website_path;
        let index_path = website_path.join("index.html");
        log::debug!("Removing html post file...");
        fs::remove_file(website_path.join(format!("posts/post-{}.html", self.post_id)))?;
        log::debug!("Reading index.html...");
        let index = fs::read_to_string(&index_path)?;

        let mut skip = 0;

        let index = index.lines().filter(|&s| {
            if s.trim_start().starts_with(&format!("<a title=\"post-{}", self.post_id)) {
                skip = 3;
                false
            } else if skip != 0 {
                skip -= 1;
                false
            } else {
                true
            }
        }).collect::<Vec<&str>>().join("\n");

        log::debug!("Replacing index.html...");
        File::create(&index_path)?.write_all(index.as_bytes())?;

        Ok(())
    }

    fn connect_md_with_template(&mut self) -> Result<String> {
        log::debug!("Reading markdown file...");
        let markdown = fs::read_to_string(&self.post_path)?;
        let html_output = Post::md_to_html(&markdown);
        log::debug!("Reading template file...");
        let template = fs::read_to_string(&self.config.blog.template_path)?;

        Ok(self.add_html_to_template(&template, &html_output)?)
    }

    fn add_html_to_template(&mut self, template: &str, html: &str) -> Result<String> {
        let date = chrono::Local::now();
        let main_title = format!(
            "<p>{}-{:>02}-{:>02}: {}</p>",
            date.year(),
            date.month(),
            date.day(),
            &self.read_info()?.title
        );

        Ok(template
            .replace("[selfblog_main_title]", &main_title)
            .replace("[selfblog_post]", &html))
    }

    fn md_to_html(markdown: &str) -> String {
        let parser = Parser::new_ext(markdown, pulldown_cmark::Options::all());
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);
        html_output
    }
}
