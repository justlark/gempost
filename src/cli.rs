use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Parser, Clone)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Args, Clone)]
pub struct Init {
    /// The directory to create the new project in
    ///
    /// This will not overwrite any files already in the directory.
    pub directory: Option<PathBuf>,
}

#[derive(Args, Clone)]
pub struct Build {
    /// The path of the gempost config file
    #[arg(short, long, value_name = "PATH", default_value = "./gempost.yaml")]
    pub config: PathBuf,
}

#[derive(Args, Clone)]
pub struct New {
    /// The URL slug of the post to create
    pub slug: String,

    /// The title of the post
    #[arg(short, long)]
    pub title: Option<String>,

    /// The path of the gempost config file
    #[arg(short, long, value_name = "PATH", default_value = "./gempost.yaml")]
    pub config: PathBuf,
}

#[derive(Args, Clone)]
pub struct NewPage {
    /// The URL slug of the page to create
    pub slug: String,

    /// The optional path beneath the pages directory to create the
    /// page at.
    #[arg(short, long, value_name = "SUBPATH")]
    pub subpath: Option<PathBuf>,

    /// The title of the post
    #[arg(short, long)]
    pub title: Option<String>,

    /// The path of the gempost config file
    #[arg(short, long, value_name = "PATH", default_value = "./gempost.yaml")]
    pub config: PathBuf,
}

#[derive(Subcommand, Clone)]
pub enum Commands {
    /// Create a new gempost project
    ///
    /// This initializes the project with some basic templates and an example gemlog post.
    Init(Init),

    /// Build your capsule
    ///
    /// This builds the gempost project in your current working directory.
    Build(Build),

    /// Create a new post
    ///
    /// This generates an empty gemtext file and YAML metadata file, automatically assigning a post ID.
    New(New),

    /// Create a new page
    ///
    /// This generates an empty gemtext file and YAML metadata file, automatically assigning a page ID.
    NewPage(NewPage)
}
