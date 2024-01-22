use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::{fs::File, path::PathBuf};

use chrono::{DateTime, FixedOffset};
use eyre::{bail, eyre, WrapErr};
use serde::Deserialize;
use url::Url;

use crate::error::Error;

const POST_FILE_EXT: &str = "gmi";
const METADATA_FILE_EXT: &str = "yaml";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct RawAuthorMetadata {
    name: String,
    email: Option<String>,
    uri: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
struct RawEntryMetadata {
    id: String,
    title: String,
    updated: String,
    summary: Option<String>,
    published: Option<String>,
    author: Option<RawAuthorMetadata>,
    rights: Option<String>,
    lang: Option<String>,
    categories: Option<Vec<String>>,
    draft: Option<bool>,
}

// This example comes from the Go standard library.
const EXAMPLE_RFC3339: &str = "2006-01-02T15:04:05Z07:00";

impl RawEntryMetadata {
    pub fn read(path: &Path) -> eyre::Result<Self> {
        let metadata_file = File::open(path)?;

        let metadata: Self = match serde_yaml::from_reader(metadata_file) {
            Ok(config) => config,
            Err(err) => bail!(Error::InvalidMetadataFile {
                path: path.to_owned(),
                reason: err.to_string(),
            }),
        };

        Ok(metadata)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorMetadata {
    pub name: String,
    pub email: Option<String>,
    pub uri: Option<String>,
}

impl From<RawAuthorMetadata> for AuthorMetadata {
    fn from(raw: RawAuthorMetadata) -> Self {
        Self {
            name: raw.name,
            email: raw.email,
            uri: raw.uri,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryMetadata {
    pub id: String,
    pub title: String,
    pub updated: DateTime<FixedOffset>,
    pub summary: Option<String>,
    pub published: Option<DateTime<FixedOffset>>,
    pub author: Option<AuthorMetadata>,
    pub rights: Option<String>,
    pub lang: Option<String>,
    pub categories: Vec<String>,
    pub draft: bool,
}

impl EntryMetadata {
    pub fn read(path: &Path) -> eyre::Result<Self> {
        let raw = RawEntryMetadata::read(path).wrap_err(format!(
            "failed reading metadata file: {}",
            path.to_string_lossy()
        ))?;

        Ok(Self {
            id: raw.id,
            title: raw.title,
            updated: DateTime::parse_from_rfc3339(&raw.updated).map_err(|_| {
                Error::InvalidMetadataFile {
                    path: path.to_owned(),
                    reason: format!(
                        "The post `updated` time must be in RFC 3339 format (e.g. {EXAMPLE_RFC3339})."
                    ),
                }
            })?,
            summary: raw.summary,
            published: raw
                .published
                .as_ref()
                .map(|published| DateTime::parse_from_rfc3339(published).map_err(|_| {
                    Error::InvalidMetadataFile {
                        path: path.to_owned(),
                        reason: format!(
                            "The post `published` time must be in RFC 3339 format (e.g. {EXAMPLE_RFC3339})."
                        ),
                    }
                }))
                .transpose()?,
            author: raw.author.map(Into::into),
            rights: raw.rights,
            lang: raw.lang,
            categories: raw.categories.unwrap_or_default(),
            // If the `draft` property is missing, we assume it's not a draft.
            draft: raw.draft.unwrap_or(false),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub metadata: EntryMetadata,
    pub body: String,
    pub url: Url,
    pub path: PathBuf,
}

pub struct PostLocation {
    pub url: Url,
    pub path: PathBuf,
}

pub struct PostLocationParams<'a> {
    pub metadata: &'a EntryMetadata,
    pub slug: &'a str,
}

// This returns `None` when either:
// - There is no filename.
// - The path is empty or the root path.
fn change_file_ext(path: &Path, new_ext: &str) -> Option<PathBuf> {
    let original_filename = path.file_stem()?;
    let new_filename = format!("{}.{}", original_filename.to_string_lossy(), new_ext);
    Some(path.parent()?.join(new_filename))
}

// Remove paths from each set that do not have an accompanying path in the other set. Emit warnings
// when this happens.
fn check_mismatched_post_files(
    post_paths: &mut HashSet<PathBuf>,
    metadata_paths: &mut HashSet<PathBuf>,
    warn_handler: impl Fn(&str),
) -> eyre::Result<()> {
    for post_path in post_paths.iter() {
        let maybe_metadata_path = match change_file_ext(post_path, METADATA_FILE_EXT) {
            Some(path) => path,
            None => bail!("This file has no filename, even though we've already checked for one. This is a bug."),
        };

        if !metadata_paths.contains(&maybe_metadata_path) {
            warn_handler(&format!(
                "This gemtext file does not have an accompanying YAML metadata file: {}",
                post_path.to_string_lossy()
            ));

            metadata_paths.remove(&maybe_metadata_path);
        }
    }

    for metadata_path in metadata_paths.iter() {
        let maybe_post_path = match change_file_ext(metadata_path, POST_FILE_EXT) {
            Some(path) => path,
            None => bail!("This file has no filename, even though we've already checked for one. This is a bug."),
        };

        if !post_paths.contains(&maybe_post_path) {
            warn_handler(&format!(
                "This YAML metadata file does not have an accompanying gemtext file: {}",
                metadata_path.to_string_lossy()
            ));

            post_paths.remove(&maybe_post_path);
        }
    }

    Ok(())
}

impl Entry {
    fn from_post_paths(
        post_paths: &HashSet<PathBuf>,
        locator: impl Fn(PostLocationParams) -> eyre::Result<PostLocation>,
    ) -> eyre::Result<Vec<Self>> {
        let mut entries = Vec::new();

        // By this point, we've already removed post paths from the set that do not have an
        // accompanying metadata file.
        for post_path in post_paths {
            let metadata_path = match change_file_ext(post_path, METADATA_FILE_EXT) {
                Some(path) => path,
                None => bail!("This file has no filename, even though we've already checked for one. This is a bug."),
            };

            let post_body = String::from_utf8(
                fs::read(post_path).wrap_err("failed reading gemtext post body")?,
            )
            .wrap_err("gemtext post body is not valid UTF-8")?;

            let post_metadata = EntryMetadata::read(&metadata_path)?;

            if post_metadata.draft {
                continue;
            }

            let post_slug = post_path
                .file_stem()
                .ok_or(eyre!(
                    "This filename does not have a file stem. This is a bug.\n{}",
                    post_path.to_string_lossy()
                ))?
                .to_string_lossy();

            let post_location = locator(PostLocationParams {
                metadata: &post_metadata,
                slug: &post_slug,
            })?;

            entries.push(Self::from(Entry {
                metadata: post_metadata,
                body: post_body,
                url: post_location.url,
                path: post_location.path,
            }));
        }

        Ok(entries)
    }

    pub fn from_posts(
        posts_dir: &Path,
        locator: impl Fn(PostLocationParams) -> eyre::Result<PostLocation>,
        warn_handler: impl Fn(&str),
    ) -> eyre::Result<Vec<Self>> {
        let file_entries = fs::read_dir(posts_dir).wrap_err("failed reading posts directory")?;

        let mut post_paths = HashSet::new();
        let mut metadata_paths = HashSet::new();

        let warn_unexpected_file_ext = |path: &Path| {
            warn_handler(&format!(
                "This is not a .gmi or .yaml file: {}",
                path.as_os_str().to_string_lossy()
            ));
        };

        for entry_result in file_entries {
            let entry_path = entry_result
                .wrap_err("failed reading posts directory")?
                .path();

            let path_ext = match entry_path.extension() {
                Some(extension) => extension,
                None => {
                    warn_unexpected_file_ext(&entry_path);
                    continue;
                }
            };

            match path_ext.to_string_lossy().as_ref() {
                POST_FILE_EXT => post_paths.insert(entry_path),
                METADATA_FILE_EXT => metadata_paths.insert(entry_path),
                _ => {
                    warn_unexpected_file_ext(&entry_path);
                    continue;
                }
            };
        }

        check_mismatched_post_files(&mut post_paths, &mut metadata_paths, warn_handler)
            .wrap_err("failed checking for mismatched post files")?;

        Self::from_post_paths(&post_paths, locator)
    }
}
