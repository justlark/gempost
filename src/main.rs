mod build;
mod cli;
mod config;
mod entry;
mod error;
mod feed;
mod init;
mod template;

use std::path::Path;
use std::process::ExitCode;

use clap::Parser;
use eyre::WrapErr;

use crate::build::build_capsule;
use crate::cli::Cli;
use crate::config::Config;
use crate::error::Error;
use crate::init::init_project;

fn run() -> eyre::Result<()> {
    let args = Cli::parse();

    match args.command {
        cli::Commands::Init(init) => {
            init_project(init.directory.as_deref().unwrap_or(Path::new(".")))
                .wrap_err("failed initializing the project")?;

            println!("Remember to edit the `gempost.yaml` to set your capsule's title and URL!")
        }
        cli::Commands::Build(build) => {
            let config = Config::read(&build.config_file)
                .wrap_err("failed reading the gempost config file")?;

            build_capsule(&config).wrap_err("failed building the capsule")?;
        }
    }

    Ok(())
}

fn main() -> eyre::Result<ExitCode> {
    color_eyre::install()?;

    if let Err(err) = run() {
        // User-facing errors should not show a stack trace.
        if let Some(user_err) = err.downcast_ref::<Error>() {
            eprintln!("{}", user_err);
            return Ok(ExitCode::FAILURE);
        }

        return Err(err);
    }

    Ok(ExitCode::SUCCESS)
}
