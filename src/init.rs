use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use eyre::{bail, WrapErr};

use crate::error::Error;

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
const POST_METADATA_FILE: &str = include_str!(r"examples\metadata.yaml");

#[cfg(not(windows))]
const POST_METADATA_FILE: &str = include_str!(r"examples/metadata.yaml");

fn put_file(abs_path: &Path, contents: &str) -> eyre::Result<()> {
    match abs_path.parent() {
        None => {
            bail!("failed creating file's parent directory because it has no parent path (this is a bug)")
        }
        Some(parent) => {
            fs::create_dir_all(parent).wrap_err("failed creating file's parent directory")?
        }
    };

    let mut example_file = match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(abs_path)
    {
        Ok(file) => file,
        Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {
            bail!(Error::ExampleFileAlreadyExists {
                path: abs_path.to_owned()
            });
        }
        Err(err) => Err(err).wrap_err("failed creating example file")?,
    };

    example_file
        .write_all(contents.as_bytes())
        .wrap_err("failed writing contents to example file")?;

    Ok(())
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
        POST_METADATA_FILE,
    )?;

    Ok(())
}
