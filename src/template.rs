use serde::{Deserialize, Serialize};

use crate::metadata::{AuthorMetadata, EntryMetadata};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorTemplateData {
    pub name: String,
    pub email: Option<String>,
    pub uri: Option<String>,
}

impl From<AuthorMetadata> for AuthorTemplateData {
    fn from(metadata: AuthorMetadata) -> Self {
        Self {
            name: metadata.name,
            email: metadata.email,
            uri: metadata.uri,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryTemplateData {
    pub uri: String,
    pub title: String,
    pub body: String,
    pub updated: String,
    pub summary: Option<String>,
    pub published: Option<String>,
    pub author: Option<AuthorTemplateData>,
    pub rights: Option<String>,
    pub lang: Option<String>,
    pub categories: Vec<String>,
}

impl EntryTemplateData {
    pub fn from_metadata(metadata: EntryMetadata, body: String, uri: String) -> Self {
        Self {
            uri,
            title: metadata.title,
            body,
            updated: metadata.updated,
            summary: metadata.summary,
            published: metadata.published,
            author: metadata.author.map(AuthorMetadata::into),
            rights: metadata.rights,
            lang: metadata.lang,
            categories: metadata.categories.unwrap_or_default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeedTemplateData {
    pub title: String,
    pub updated: String,
    pub subtitle: Option<String>,
    pub rights: Option<String>,
    pub author: Option<AuthorTemplateData>,
    pub entries: Vec<EntryTemplateData>,
}
