use clap::{Args, Parser, Subcommand};
use log::LevelFilter;
use anyhow::Result;

#[derive(Parser, Debug)]
#[clap(
    author = "Writted by reticulis <reticulis@protonmail.com>",
    version = "0.0.0",
    about = "Create your own simple blog!",
)]
struct Cli {
    #[clap(subcommand)]
    command: Subcommands,

    #[clap(short, long, default_value_t = 1)]
    debug: u8,
}

#[derive(Debug, Subcommand)]
enum Subcommands {
    /// HTTP server initialization
    Init,
    /// Create a new draft post
    NewPost {
        title: String
    },
    /// Check and mark the post as ready to publish
    Ready,
    /// Publish your post to blog
    Publish
}

fn main() -> Result<()> {
    // env_logger::builder().filter_level(LevelFilter::).try_init();

    let mut builder = env_logger::builder();

    let args = Cli::parse();

    match args.debug {
        0 => {
            println!("Debug mode is off!");
            builder.filter_level(LevelFilter::Off).try_init()?
        }
        1 => {
            println!("Debug mode is in info only!");
            builder.filter_level(LevelFilter::Info).try_init()?
        }
        2 => {
            println!("Debug mode is on!");
            builder.filter_level(LevelFilter::Debug).try_init()?
        }
        _ => {
            println!("Invalid Debug mode level!");
            std::process::exit(1)
        }
    }

    match args.command {
        Subcommands::Init => {
            log::info!("HTTP server initialization...");
        },
        Subcommands::NewPost { title } => {
            log::info!("Creating a new draft post...")
        },
        Subcommands::Ready => {
            log::info!("Checking and marking the post as ready to publish...")
        },
        Subcommands::Publish => {
            log::info!("Publishing your post to blog...")
        }
    }

    Ok(())
}
