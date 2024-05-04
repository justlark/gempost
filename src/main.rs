mod build;
mod cli;
mod config;
mod entry;
mod entry_util;
mod error;
mod feed;
mod init;
mod new;
mod page;
mod page_entry;
mod template;

use std::path::Path;
use std::process::ExitCode;

use clap::Parser;
use eyre::WrapErr;
use new::{create_new_page, create_new_post};

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
            let config =
                Config::read(&build.config).wrap_err("failed reading the gempost config file")?;

            build_capsule(&config).wrap_err("failed building the capsule")?;
        }
        cli::Commands::New(new) => {
            let config =
                Config::read(&new.config).wrap_err("failed reading the gempost config file")?;

            create_new_post(&config.posts_dir, &new.slug, new.title.as_deref())
                .wrap_err("failed creating new gemlog post")?;
        }
        cli::Commands::NewPage(new_page) => {
            let config = Config::read(&new_page.config)
                .wrap_err("failed reading the gempost config file")?;

            create_new_page(
                &config.pages_dir,
                &new_page.slug,
                new_page.title.as_deref(),
                new_page.subpath.as_deref(),
            )
            .wrap_err("failed creating new gemlog post")?;
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
