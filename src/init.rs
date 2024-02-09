use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use chrono::{Local, SecondsFormat};
use eyre::{bail, WrapErr};
use tera::{Context, Tera};
use uuid::Uuid;

use crate::error::Error;

// We need to use conditional compilation here because `include_str` interprets the path in a
// platform-specific way at compile time.
//
// https://doc.rust-lang.org/std/macro.include_str.html

#[cfg(windows)]
const CONFIG_FILE: &str = include_str!(r"examples\gempost.yaml");

#[cfg(not(windows))]
const CONFIG_FILE: &str = include_str!(r"examples/gempost.yaml");

#[cfg(windows)]
const CAPSULE_INDEX_FILE: &str = include_str!(r"examples\index.gmi");

#[cfg(not(windows))]
const CAPSULE_INDEX_FILE: &str = include_str!(r"examples/index.gmi");

#[cfg(windows)]
const INDEX_TEMPLATE_FILE: &str = include_str!(r"examples\index.tera");

#[cfg(not(windows))]
const INDEX_TEMPLATE_FILE: &str = include_str!(r"examples/index.tera");

#[cfg(windows)]
const POST_TEMPLATE_FILE: &str = include_str!(r"examples\post.tera");

#[cfg(not(windows))]
const POST_TEMPLATE_FILE: &str = include_str!(r"examples/post.tera");

#[cfg(windows)]
const GEMLOG_POST_FILE: &str = include_str!(r"examples\post.gmi");

#[cfg(not(windows))]
const GEMLOG_POST_FILE: &str = include_str!(r"examples/post.gmi");

#[cfg(windows)]
const POST_METADATA_FILE: &str = include_str!(r"examples\metadata.yaml.tera");

#[cfg(not(windows))]
const POST_METADATA_FILE: &str = include_str!(r"examples/metadata.yaml.tera");

fn put_file(file: &Path, contents: &str) -> eyre::Result<()> {
    match file.parent() {
        None => {
            bail!("failed creating file's parent directory because it has no parent path (this is a bug)")
        }
        Some(parent) => {
            fs::create_dir_all(parent).wrap_err("failed creating file's parent directory")?
        }
    };

    let mut example_file = match OpenOptions::new().write(true).create_new(true).open(file) {
        Ok(file) => file,
        Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {
            bail!(Error::ExampleFileAlreadyExists {
                path: file.to_owned()
            });
        }
        Err(err) => Err(err).wrap_err("failed creating example file")?,
    };

    example_file
        .write_all(contents.as_bytes())
        .wrap_err("failed writing contents to example file")?;

    Ok(())
}

fn generate_example_metadata_file(template: &str) -> eyre::Result<String> {
    let mut tera = Tera::default();

    tera.add_raw_template("metadata", template)
        .wrap_err("Example metadata file template is invalid. This is a bug.")?;

    let mut context = Context::new();
    context.insert("id", &format!("urn:uuid:{}", Uuid::new_v4()));
    context.insert(
        "timestamp",
        &Local::now().to_rfc3339_opts(SecondsFormat::Secs, false),
    );

    tera.render("metadata", &context)
        .wrap_err("Failed to render example metadata file template. This is a bug.")
}

pub fn init_project(dir: &Path) -> eyre::Result<()> {
    put_file(&dir.join("gempost.yaml"), CONFIG_FILE)?;
    put_file(&dir.join("static").join("index.gmi"), CAPSULE_INDEX_FILE)?;
    put_file(
        &dir.join("templates").join("index.tera"),
        INDEX_TEMPLATE_FILE,
    )?;
    put_file(&dir.join("templates").join("post.tera"), POST_TEMPLATE_FILE)?;
    put_file(&dir.join("posts").join("hello-world.gmi"), GEMLOG_POST_FILE)?;
    put_file(
        &dir.join("posts").join("hello-world.yaml"),
        &generate_example_metadata_file(POST_METADATA_FILE)?,
    )?;

    Ok(())
}
