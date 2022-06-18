use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(
    author = "Writted by reticulis <reticulis@protonmail.com>",
    version = "0.0.0",
    about = "Create your own simple blog!",
)]
struct Cli {
    #[clap(subcommand)]
    command: Subcommands,
}

#[derive(Debug, Subcommand)]
enum Subcommands {
    /// Server initialization
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

fn main() {
    env_logger::init();

    let args = Cli::parse();

    match args.command {
        Subcommands::Init => {
            log::debug!("Server initialization...")
        },
        Subcommands::NewPost { title } => {
            log::debug!("Creating a new draft post...")
        },
        Subcommands::Ready => {
            log::debug!("Checking and marking the post as ready to publish...")
        },
        Subcommands::Publish => {
            log::debug!("Publishing your post to blog...")
        }
    }
}
