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
    pub public_dir: PathBuf,
    #[serde(default = "defaults::static_dir")]
    pub static_dir: PathBuf,
    #[serde(default = "defaults::posts_dir")]
    pub posts_dir: PathBuf,
    #[serde(default = "defaults::index_template_file")]
    pub index_template_file: PathBuf,
    #[serde(default = "defaults::post_template_file")]
    pub post_template_file: PathBuf,
    #[serde(default = "defaults::post_path")]
    pub post_path: String,
    #[serde(default = "defaults::index_path")]
    pub index_path: String,
    #[serde(default = "defaults::feed_path")]
    pub feed_path: String,
    pub title: String,
    pub uri: String,
    pub subtitle: Option<String>,
    pub rights: Option<String>,
    pub author: Option<AuthorMetadata>,
}

mod defaults {
    use std::path::PathBuf;

    pub fn public_dir() -> PathBuf {
        PathBuf::from("./public/")
    }

    pub fn static_dir() -> PathBuf {
        PathBuf::from("./static/")
    }

    pub fn posts_dir() -> PathBuf {
        PathBuf::from("./posts/")
    }

    pub fn index_template_file() -> PathBuf {
        PathBuf::from("./templates/index.tera")
    }

    pub fn post_template_file() -> PathBuf {
        PathBuf::from("./templates/post.tera")
    }

    pub fn post_path() -> String {
        String::from("/posts/{{ slug }}")
    }

    pub fn index_path() -> String {
        String::from("/posts/index.gmi")
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
