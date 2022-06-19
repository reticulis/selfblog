use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use daemonize::Daemonize;
use log::LevelFilter;
use std::fs::File;
use std::io::Read;
use std::{fs, thread};
use std::time::Duration;

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

fn main() -> Result<()> {
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
        Subcommands::Start => {
            log::info!("HTTP server initialization...");
            let stdout = File::create("/tmp/selfblog.out").unwrap();
            let stderr = File::create("/tmp/selfblog.err").unwrap();

            let daemonize = Daemonize::new()
                .pid_file("/tmp/selfblog-daemon.pid")
                .stdout(stdout)
                .stderr(stderr);

            match daemonize.start() {
                Ok(_) => {
                    unimplemented!()
                }
                Err(e) => log::error!("Error: {}", e),
            }
        }
        Subcommands::Stop => {
            log::debug!("Reading \"/tmp/selfblog-daemon.pid\"...");
            let pid = fs::read_to_string("/tmp/selfblog-daemon.pid")?;
            log::info!("Stoping server...");
            std::process::Command::new("kill").arg(pid).spawn()?;
        }
        Subcommands::NewPost { title } => {
            log::info!("Creating a new draft post...")
        }
        Subcommands::Ready => {
            log::info!("Checking and marking the post as ready to publish...")
        }
        Subcommands::Publish => {
            log::info!("Publishing your post to blog...")
        }
    }

    Ok(())
}
