mod cli;
mod config;
mod error;
mod metadata;
mod template;

use std::process::ExitCode;

use clap::Parser;
use cli::Cli;
use error::Error;

fn run() -> eyre::Result<()> {
    let _ = Cli::parse();

    // TODO

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
