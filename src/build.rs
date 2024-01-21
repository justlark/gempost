use std::fs;
use std::path::{Path, PathBuf};

use eyre::{bail, WrapErr};

use crate::config::Config;
use crate::error::Error;
use crate::template::FeedTemplateData;

const FEED_TEMPLATE: &str = include_str!("atom.xml.tera");

fn url_path_to_file_path(file_path: &Path, url_path: &str) -> PathBuf {
    file_path.join(PathBuf::from_iter(
        url_path.split('/').filter(|segment| !segment.is_empty()),
    ))
}

fn copy_dir(src: &Path, dest: &Path) -> eyre::Result<()> {
    fs::create_dir_all(dest).wrap_err("failed creating dest directory")?;

    let src_entries = fs::read_dir(src).wrap_err("failed reading directory contents")?;

    for src_entry_result in src_entries {
        let src_entry = src_entry_result.wrap_err("failed reading directory entry")?;

        let file_type = src_entry.file_type().wrap_err("failed reading file type")?;

        let src_path = src_entry.path();
        let dest_path = dest.join(src_path.strip_prefix(src)?);

        if file_type.is_file() {
            if dest_path
                .try_exists()
                .wrap_err("failed checking if dest file exists")?
            {
                bail!(Error::StaticPathConflict { path: dest_path });
            }

            fs::copy(src_path, dest_path).wrap_err("failed copying regular file")?;
        } else if file_type.is_dir() {
            fs::create_dir(&dest_path).wrap_err("failed creating new directory in dest dir")?;

            copy_dir(&src_path, &dest_path)?;
        } else if file_type.is_symlink() {
            let link_dest = fs::read_link(src_path).wrap_err("failed reading link dest")?;

            if !cfg!(target_family = "unix") {
                bail!(
                    "Symlinks in the static directory are only supported on Unix-like platforms."
                );
            }

            #[cfg(target_family = "unix")]
            std::os::unix::fs::symlink(link_dest, dest_path).wrap_err("failed creating symlink")?;
        } else {
            bail!("unrecognized file type in static directory");
        }
    }

    Ok(())
}

pub fn build_capsule(config: &Config) -> eyre::Result<()> {
    let warn_handler = |msg: &str| eprintln!("{}", msg);

    let feed_data = FeedTemplateData::from_config(config, warn_handler)
        .wrap_err("failed parsing config file")?;

    // Generate index page.

    let index_page_path = url_path_to_file_path(&config.public_dir, &config.index_path);
    feed_data
        .render_index(&config.index_template_file, &index_page_path)
        .wrap_err("failed rendering index page")?;

    // Generate Atom feed.

    let feed_path = url_path_to_file_path(&config.public_dir, &config.feed_path);
    feed_data
        .render_feed(FEED_TEMPLATE, &feed_path)
        .wrap_err("failed rendering Atom feed")?;

    // Generate individual posts.

    for entry in &feed_data.entries {
        let post_path = url_path_to_file_path(&config.public_dir, &entry.path);

        entry
            .render(&config.post_template_file, &post_path)
            .wrap_err(format!(
                "failed rendering post: {}",
                post_path.to_string_lossy()
            ))?;
    }

    // Copy over static content

    copy_dir(&config.static_dir, &config.public_dir).wrap_err(
        "failed copying static content from the static directory to the public directory",
    )?;

    Ok(())
}
