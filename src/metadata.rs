use std::time::SystemTime;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorMetadata {
    name: String,
    email: Option<String>,
    uri: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PostMetadata {
    id: String,
    url: String,
    title: String,
    summary: Option<String>,
    published: SystemTime,
    updated: Option<SystemTime>,
    author: Option<AuthorMetadata>,
}
