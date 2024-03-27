use std::path::PathBuf;

use thiserror::Error;

/// A error type for user-facing errors.
///
/// This type represents errors expected in common usage of the program that should trigger a
/// readable error message instead of a stack trace.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    #[error("There is no config file at `{path}`.")]
    NonexistentConfigFile { path: PathBuf },

    #[error("There is a problem with the config file at `{path}`.\n\n{reason}")]
    InvalidConfigFile { path: PathBuf, reason: String },

    #[error("There is a problem with the post metadata file at `{path}`.\n\n{reason}")]
    InvalidMetadataFile { path: PathBuf, reason: String },

    #[error("You cannot initialize this directory as a gempost project because this file already exists: {path}")]
    ExampleFileAlreadyExists { path: PathBuf },

    #[error("There is already a post with this slug: {slug}")]
    PostAlreadyExists { slug: String },

    #[error("There was an issue generating the index page.\n\n{reason}")]
    InvalidIndexPageTemplate { reason: String },

    #[error("There was an issue generating a post page.\n\n{reason}")]
    InvalidPostPageTemplate { path: PathBuf, reason: String },

    #[error("There was an issue generating a templated page.\n\n{reason}")]
    InvalidPageTemplate { path: PathBuf, reason: String },

    #[error("The post path template in your gempost.yaml is invalid.\n\nTemplate: `{template}`\n\n{reason}")]
    InvalidPostPath { template: String, reason: String },

    #[error("The capsule URL you provided is not a valid URL: {url}")]
    InvalidCapsuleUrl { url: String },
}
