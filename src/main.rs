mod server;
mod subcommands;

use anyhow::Result;
use clap::{Parser, Subcommand};
use log::LevelFilter;

#[derive(Parser, Debug)]
#[clap(
    author = "Writted by reticulis <reticulis@protonmail.com>",
    version = "0.0.0",
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
    Start,
    /// Stop HTTP server
    Stop,
    /// Create a new draft post
    NewPost { title: String },
    /// Check and mark the post as ready to publish
    Ready,
    /// Publish your post to blog
    Publish,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let mut builder = env_logger::builder();

    let args = Cli::parse();

    match args.debug {
        0 => {
            builder.filter_level(LevelFilter::Off).try_init()?;
        }
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
        Subcommands::Init { config: _ } => {
            unimplemented!()
        }
        Subcommands::Start => subcommands::start().await?,
        Subcommands::Stop => subcommands::stop()?,
        Subcommands::NewPost { title: _ } => {
            log::info!("Creating a new draft post...");
            unimplemented!();
        }
        Subcommands::Ready => {
            log::info!("Checking and marking the post as ready to publish...");
            unimplemented!();
        }
        Subcommands::Publish => {
            log::info!("Publishing your post to blog...");
            unimplemented!();
        }
    }

    Ok(())
}
