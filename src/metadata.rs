use std::fs::File;
use std::path::Path;

use chrono::DateTime;
use eyre::bail;
use serde::{Deserialize, Serialize};
use url::Url;

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
    pub categories: Option<Vec<String>>,
    pub draft: bool,
}

fn is_uri(s: &str) -> bool {
    Url::parse(s).is_ok()
}

fn is_rfc3339(s: &str) -> bool {
    DateTime::parse_from_rfc3339(s).is_ok()
}

// This example comes from the Go standard library.
const EXAMPLE_RFC3339: &str = "2006-01-02T15:04:05Z07:00";

impl EntryMetadata {
    pub fn read(path: &Path) -> eyre::Result<Self> {
        let metadata_file = File::open(path)?;

        let metadata: Self = match serde_yaml::from_reader(metadata_file) {
            Ok(config) => config,
            Err(err) => bail!(Error::InvalidMetadataFile {
                path: path.to_owned(),
                reason: err.to_string(),
            }),
        };

        metadata.validate(path)?;

        Ok(metadata)
    }

    fn validate(&self, path: &Path) -> eyre::Result<()> {
        if !is_uri(&self.id) {
            bail!(Error::InvalidMetadataFile {
                path: path.to_owned(),
                reason: String::from("The post `id` must be a valid URI."),
            })
        }

        if !is_rfc3339(&self.updated) {
            bail!(Error::InvalidMetadataFile {
                path: path.to_owned(),
                reason: format!(
                    "The post `updated` time must be in RFC3339 format (e.g. {EXAMPLE_RFC3339})."
                ),
            })
        }

        if let Some(published) = &self.published {
            if !is_rfc3339(published) {
                bail!(Error::InvalidMetadataFile {
                    path: path.to_owned(),
                    reason: format!("The post `published` time must be in RFC3339 format (e.g. {EXAMPLE_RFC3339})."),
                })
            }
        }

        Ok(())
    }
}
