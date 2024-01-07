use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Clone)]
#[command(author, version, about)]
pub struct Cli {
    /// The path of the config file.
    #[arg(long, value_name = "PATH", default_value = "./gempost.yaml")]
    pub config_file: PathBuf,
}
