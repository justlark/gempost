use std::fs::File;
use std::path::Path;

use eyre::bail;
use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorMetadata {
    pub name: String,
    pub email: Option<String>,
    pub uri: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryMetadata {
    pub id: String,
    pub title: String,
    pub updated: String,
    pub summary: Option<String>,
    pub published: Option<String>,
    pub author: Option<AuthorMetadata>,
    pub rights: Option<String>,
    pub lang: Option<String>,
    pub draft: bool,
}

impl EntryMetadata {
    pub fn read(path: &Path) -> eyre::Result<Self> {
        let metadata_file = File::open(path)?;

        match serde_yaml::from_reader(metadata_file) {
            Ok(config) => Ok(config),
            Err(err) => bail!(Error::InvalidMetadataFile {
                path: path.to_owned(),
                reason: err.to_string(),
            }),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeedMetadata {
    pub id: String,
    pub title: String,
    pub page_uri: String,
    pub feed_uri: String,
    pub subtitle: Option<String>,
    pub rights: Option<String>,
    pub author: Option<AuthorMetadata>,
    pub updated: String,
    pub entries: Vec<EntryMetadata>,
}
