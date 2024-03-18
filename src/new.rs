use std::fs::{create_dir_all, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use chrono::{Local, SecondsFormat};
use eyre::{bail, WrapErr};
use tera::{Context, Tera};
use uuid::Uuid;

use crate::error::Error;

const METADATA_TEMPLATE: &str = include_str!("metadata.yaml.tera");

fn generate_metadata_file(template: &str, title: Option<&str>) -> eyre::Result<String> {
    let mut tera = Tera::default();

    tera.add_raw_template("metadata", template)
        .wrap_err("New metadata file template is invalid. This is a bug.")?;

    let mut context = Context::new();
    context.insert("id", &format!("urn:uuid:{}", Uuid::new_v4()));
    context.insert("title", title.unwrap_or_default());
    context.insert(
        "timestamp",
        &Local::now().to_rfc3339_opts(SecondsFormat::Secs, false),
    );

    tera.render("metadata", &context)
        .wrap_err("Failed to render new metadata file template. This is a bug.")
}

pub fn create_new_post(posts_dir: &Path, slug: &str, title: Option<&str>) -> eyre::Result<()> {
    let gemtext_path = posts_dir.join(format!("{slug}.gmi"));
    let metadata_path = posts_dir.join(format!("{slug}.yaml"));

    // Generate an empty gemtext file.

    match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(gemtext_path)
    {
        Ok(file) => file,
        Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {
            bail!(Error::PostAlreadyExists {
                slug: slug.to_owned()
            });
        }
        Err(err) => Err(err).wrap_err("failed creating new post gemtext file")?,
    };

    // Generate a metadata YAML file.

    let mut metadata_file = match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(metadata_path)
    {
        Ok(file) => file,
        Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {
            bail!(Error::PostAlreadyExists {
                slug: slug.to_owned()
            });
        }
        Err(err) => Err(err).wrap_err("failed creating new post metadata file")?,
    };

    let metadata_file_contents = generate_metadata_file(METADATA_TEMPLATE, title)
        .wrap_err("failed generating contents for new post metadata file")?;

    metadata_file
        .write_all(metadata_file_contents.as_bytes())
        .wrap_err("failed writing contents to new post metadata file")?;

    Ok(())
}

pub fn create_new_page(
    pages_dir: &Path,
    slug: &str,
    title: Option<&str>,
    subpath: Option<&Path>,
) -> eyre::Result<()> {
    // By default, files are created at the root. But they may be
    // created at an optional subpath under the root.
    let pages_dir = subpath
        .map(|sp| pages_dir.join(sp))
        .unwrap_or_else(|| pages_dir.to_path_buf());

    create_dir_all(&pages_dir).wrap_err("failed to create the requested page directory")?;

    let gemtext_path = pages_dir.join(format!("{slug}.gmi"));
    let metadata_path = pages_dir.join(format!("{slug}.yaml"));

    // Generate an empty gemtext file.

    match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(gemtext_path)
    {
        Ok(file) => file,
        Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {
            bail!(Error::PostAlreadyExists {
                slug: slug.to_owned()
            });
        }
        Err(err) => Err(err).wrap_err("failed creating new page gemtext file")?,
    };

    // Generate a metadata YAML file.

    let mut metadata_file = match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(metadata_path)
    {
        Ok(file) => file,
        Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {
            bail!(Error::PostAlreadyExists {
                slug: slug.to_owned()
            });
        }
        Err(err) => Err(err).wrap_err("failed creating new page metadata file")?,
    };

    let metadata_file_contents = generate_metadata_file(METADATA_TEMPLATE, title)
        .wrap_err("failed generating contents for new page metadata file")?;

    metadata_file
        .write_all(metadata_file_contents.as_bytes())
        .wrap_err("failed writing contents to new page metadata file")?;

    Ok(())
}
