use std::io;
use std::path::PathBuf;
use std::{fs::File, path::Path};

use eyre::{bail, WrapErr};
use serde::Deserialize;
use url::Url;

use crate::error::Error;

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct RawAuthorConfig {
    pub name: String,
    pub email: Option<String>,
    pub uri: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
struct RawConfig {
    #[serde(default = "defaults::public_dir")]
    public_dir: PathBuf,
    #[serde(default = "defaults::static_dir")]
    static_dir: PathBuf,
    #[serde(default = "defaults::posts_dir")]
    posts_dir: PathBuf,
    #[serde(default = "defaults::index_template_file")]
    index_template_file: PathBuf,
    #[serde(default = "defaults::post_template_file")]
    post_template_file: PathBuf,
    #[serde(default = "defaults::post_path")]
    post_path: String,
    #[serde(default = "defaults::index_path")]
    index_path: String,
    #[serde(default = "defaults::feed_path")]
    feed_path: String,
    title: String,
    url: String,
    subtitle: Option<String>,
    rights: Option<String>,
    author: Option<RawAuthorConfig>,
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

impl RawConfig {
    fn read(path: &Path) -> eyre::Result<Self> {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorConfig {
    pub name: String,
    pub email: Option<String>,
    pub uri: Option<String>,
}

impl From<RawAuthorConfig> for AuthorConfig {
    fn from(raw: RawAuthorConfig) -> Self {
        Self {
            name: raw.name,
            email: raw.email,
            uri: raw.uri,
        }
    }
}

#[derive(Debug)]
pub struct Config {
    pub public_dir: PathBuf,
    pub static_dir: PathBuf,
    pub posts_dir: PathBuf,
    pub index_template_file: PathBuf,
    pub post_template_file: PathBuf,
    pub post_path: String,
    pub index_path: String,
    pub feed_path: String,
    pub title: String,
    pub url: Url,
    pub subtitle: Option<String>,
    pub rights: Option<String>,
    pub author: Option<AuthorConfig>,
}

impl Config {
    pub fn read(path: &Path) -> eyre::Result<Self> {
        let raw = RawConfig::read(path).wrap_err("failed reading config file")?;

        Ok(Self {
            public_dir: raw.public_dir,
            static_dir: raw.static_dir,
            posts_dir: raw.posts_dir,
            index_template_file: raw.index_template_file,
            post_template_file: raw.post_template_file,
            post_path: raw.post_path,
            index_path: raw.index_path,
            feed_path: raw.feed_path,
            title: raw.title,
            url: Url::parse(&raw.url).map_err(|_| Error::InvalidCapsuleUrl { url: raw.url })?,
            subtitle: raw.subtitle,
            rights: raw.rights,
            author: raw.author.map(Into::into),
        })
    }
}
