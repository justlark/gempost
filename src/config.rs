use std::io;
use std::path::PathBuf;
use std::{fs::File, path::Path};

use eyre::bail;
use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::metadata::AuthorMetadata;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "defaults::public_dir")]
    public_dir: PathBuf,
    #[serde(default = "defaults::static_dir")]
    static_dir: PathBuf,
    #[serde(default = "defaults::index_template_file")]
    index_template_file: PathBuf,
    #[serde(default = "defaults::post_template_file")]
    post_template_file: PathBuf,
    #[serde(default = "defaults::post_path")]
    post_path: String,
    #[serde(default = "defaults::feed_path")]
    feed_path: String,
    title: String,
    uri: String,
    subtitle: Option<String>,
    rights: Option<String>,
    author: Option<AuthorMetadata>,
}

mod defaults {
    use std::path::PathBuf;

    pub fn public_dir() -> PathBuf {
        PathBuf::from("./public")
    }

    pub fn static_dir() -> PathBuf {
        PathBuf::from("./static")
    }

    pub fn index_template_file() -> PathBuf {
        PathBuf::from("./index.tera")
    }

    pub fn post_template_file() -> PathBuf {
        PathBuf::from("./post.tera")
    }

    pub fn post_path() -> String {
        String::from("/posts/{{ slug }}")
    }

    pub fn feed_path() -> String {
        String::from("/posts/atom.xml")
    }
}

impl Config {
    pub fn read(path: &Path) -> eyre::Result<Self> {
        let config_file = match File::open(path) {
            Ok(file) => file,
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                bail!(Error::NonexistentConfigFile {
                    path: path.to_owned(),
                })
            }
            Err(err) => bail!(err),
        };

        match serde_yaml::from_reader(config_file) {
            Ok(config) => Ok(config),
            Err(err) => bail!(Error::InvalidConfigFile {
                path: path.to_owned(),
                reason: err.to_string(),
            }),
        }
    }
}
