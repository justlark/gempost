use std::io;
use std::path::PathBuf;
use std::{fs::File, path::Path};

use eyre::bail;
use serde::Deserialize;
use url::Url;

use crate::error::Error;
use crate::metadata::AuthorMetadata;

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
    author: Option<AuthorMetadata>,
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
    pub author: Option<AuthorMetadata>,
}

impl TryFrom<RawConfig> for Config {
    type Error = eyre::Report;

    fn try_from(raw: RawConfig) -> eyre::Result<Self> {
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
            author: raw.author,
        })
    }
}

impl Config {
    pub fn read(path: &Path) -> eyre::Result<Self> {
        RawConfig::read(path)?.try_into()
    }
}
