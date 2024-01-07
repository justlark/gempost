use std::fs::File;
use std::path::Path;
use std::time::SystemTime;

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
pub struct PostMetadata {
    id: String,
    title: String,
    summary: Option<String>,
    published: SystemTime,
    updated: Option<SystemTime>,
    author: Option<AuthorMetadata>,
    draft: Option<bool>,
}

impl PostMetadata {
    pub fn is_draft(&self) -> bool {
        matches!(self.draft, Some(true))
    }

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
