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
    #[serde(default = "defaults::url_pattern")]
    url_pattern: String,
    title: String,
    url: String,
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

    pub fn url_pattern() -> String {
        String::from("/posts/{{ slug }}")
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
