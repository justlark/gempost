use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use eyre::{bail, WrapErr};

use crate::config::Config;
use crate::feed::Feed;
use crate::template::{EntryTemplateData, FeedTemplateData};

const FEED_TEMPLATE: &str = include_str!("atom.xml.tera");

fn url_to_filepath(base_path: &Path, url_path: &str) -> PathBuf {
    base_path.join(PathBuf::from_iter(
        url_path.split('/').filter(|segment| !segment.is_empty()),
    ))
}

// Recursively copy a directory.
fn copy_dir(src: &Path, dest: &Path) -> eyre::Result<()> {
    fs::create_dir_all(dest).wrap_err("failed creating dest directory")?;

    let src_entries = fs::read_dir(src).wrap_err("failed reading directory contents")?;

    for src_entry_result in src_entries {
        let src_entry = src_entry_result.wrap_err("failed reading directory entry")?;

        let file_type = src_entry.file_type().wrap_err("failed reading file type")?;

        let src_path = src_entry.path();
        let dest_path = dest.join(src_path.strip_prefix(src)?);

        if file_type.is_file() {
            // Truncate the dest file if it already exists.
            fs::copy(&src_path, &dest_path).wrap_err("failed copying regular file")?;
        } else if file_type.is_dir() {
            // Don't fail if the dest dir already exists.
            fs::create_dir_all(&dest_path).wrap_err("failed creating new directory in dest dir")?;

            // Recursively copy contents.
            copy_dir(&src_path, &dest_path)?;
        } else if file_type.is_symlink() {
            let link_dest = fs::read_link(&src_path).wrap_err("failed reading link dest")?;

            if !cfg!(target_family = "unix") {
                bail!(
                    "Symlinks in the static directory are only supported on Unix-like platforms."
                );
            }

            // Overwrite the original symlink if it exists. Do nothing if it does not.
            match fs::remove_file(&dest_path) {
                Err(err) if err.kind() != io::ErrorKind::NotFound => Err(err)
                    .wrap_err("failed removing original symlink so we can create a new one")?,
                _ => {}
            }

            #[cfg(target_family = "unix")]
            std::os::unix::fs::symlink(link_dest, &dest_path)
                .wrap_err("failed creating symlink in dest dir")?;
        } else {
            bail!("There is a file in the static directory which is not a regular file, directory, or symbolic link.");
        }
    }

    Ok(())
}

pub fn build_capsule(config: &Config) -> eyre::Result<()> {
    let warn_handler = |msg: &str| eprintln!("Warning: {}", msg);

    let feed = Feed::from_config(config, warn_handler).wrap_err("failed parsing config file")?;
    let feed_data = FeedTemplateData::from(feed.clone());

    // Delete the public dir. We do this because static files might have been removed since the
    // last build, and posts might have been removed or converted to drafts. It's easier to just
    // start with a new empty directory.

    match fs::remove_dir_all(&config.public_dir) {
        // The public dir not existing is not an error.
        Err(err) if err.kind() != io::ErrorKind::NotFound => {
            Err(err).wrap_err("failed removing the public directory")?
        }
        _ => {}
    }

    fs::create_dir_all(&config.public_dir).wrap_err("failed creating the public directory")?;

    // Generate the index page.

    let index_page_path = url_to_filepath(&config.public_dir, &config.index_path);
    feed_data
        .render_index(&config.index_template_file, &index_page_path)
        .wrap_err("failed rendering index page")?;

    // Generate the Atom feed.

    let feed_path = url_to_filepath(&config.public_dir, &config.feed_path);
    feed_data
        .render_feed(FEED_TEMPLATE, &feed_path)
        .wrap_err("failed rendering Atom feed")?;

    // Generate the individual posts.

    for entry in feed.entries {
        let post_path = config.public_dir.join(&entry.path);

        EntryTemplateData::from(entry)
            .render(&feed_data, &config.post_template_file, &post_path)
            .wrap_err(format!(
                "failed rendering post: {}",
                post_path.to_string_lossy()
            ))?;
    }

    // Copy over static content. This clobbers any files generated in previous steps.

    copy_dir(&config.static_dir, &config.public_dir)
        .wrap_err("failed copying static content to the public directory")?;

    Ok(())
}
