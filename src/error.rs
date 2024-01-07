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
}