mod cli;
mod config;
mod error;
mod init;
mod metadata;
mod template;

use std::path::Path;
use std::process::ExitCode;

use clap::Parser;
use cli::Cli;
use error::Error;

use crate::init::init_project;

fn run() -> eyre::Result<()> {
    let args = Cli::parse();

    match args.command {
        cli::Commands::Init(init) => {
            init_project(init.directory.as_deref().unwrap_or(Path::new(".")))?;

            println!("Remember to edit the `gempost.yaml` to set your capsule's title and URI!")
        }
        cli::Commands::Build(_) => todo!(),
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
