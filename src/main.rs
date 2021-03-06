mod config;
mod post;
mod subcommands;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use log::LevelFilter;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(
    author = "reticulis <reticulis@protonmail.com>",
    version = "0.5.1",
    about = "Create your own simple blog!"
)]
struct Cli {
    #[clap(subcommand)]
    command: Subcommands,

    #[clap(short, long, default_value_t = 1)]
    debug: u8,
}

#[derive(Debug, Subcommand)]
enum Subcommands {
    /// Create required files
    Init { config: String },
    /// Start HTTP server
    #[clap(subcommand)]
    Start(Start),
    /// Stop HTTP server
    Stop,
    /// Create a new draft post
    NewPost { title: String, description: String },
    /// Check and mark the post as ready to publish
    Ready,
    /// Publish your post to blog
    Publish,
    /// Update your post
    Update { post_id: usize },
    /// Delete post
    Delete { post_id: usize },
}

#[derive(Debug, Subcommand)]
pub enum Start {
    Gemini,
    Www,
    All,
}

fn main() -> Result<()> {
    let mut builder = env_logger::builder();

    let args = Cli::parse();

    match args.debug {
        0 => builder.filter_level(LevelFilter::Off).try_init()?,
        1 => {
            builder.filter_level(LevelFilter::Info).try_init()?;
            log::info!("Debug mode is in info only!");
        }
        2 => {
            builder.filter_level(LevelFilter::Debug).try_init()?;
            log::debug!("Debug mode is on!");
        }
        _ => {
            eprintln!("Invalid Debug mode level!");
            std::process::exit(1)
        }
    }

    match args.command {
        Subcommands::Init { config } => subcommands::init(&config)?,
        Subcommands::Start(option) => subcommands::start(option)?,
        Subcommands::Stop => subcommands::stop()?,
        Subcommands::NewPost { title, description } => subcommands::new_post(title, description)?,
        Subcommands::Ready => subcommands::ready()?,
        Subcommands::Publish => subcommands::publish()?,
        Subcommands::Update { post_id } => subcommands::update(post_id)?,
        Subcommands::Delete { post_id } => subcommands::delete(post_id)?,
    }

    Ok(())
}

pub fn home() -> Result<PathBuf> {
    dirs::home_dir().with_context(|| "Failed getting home dir path!")
}
