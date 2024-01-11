use std::fs::File;
use std::path::Path;

use eyre::bail;
use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorMetadata {
    name: String,
    email: Option<String>,
    uri: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryMetadata {
    id: String,
    uri: String,
    title: String,
    updated: String,
    summary: Option<String>,
    published: Option<String>,
    author: Option<AuthorMetadata>,
    rights: Option<String>,
    lang: Option<String>,
    draft: bool,
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
    id: String,
    title: String,
    page_uri: String,
    feed_uri: String,
    subtitle: Option<String>,
    rights: Option<String>,
    author: Option<AuthorMetadata>,
    updated: String,
    entries: Vec<EntryMetadata>,
}
