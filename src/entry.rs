use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::{fs::File, path::PathBuf};

use chrono::{DateTime, FixedOffset};
use eyre::{bail, eyre, WrapErr};
use serde::Deserialize;
use serde_yaml::{Mapping as YamlMapping, Value as YamlValue};
use url::Url;

use crate::entry_util::{check_mismatched_files, PathPair, METADATA_FILE_EXT, POST_FILE_EXT};
use crate::error::Error;

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
    pub fn read_from_value(value: &YamlValue, path: &Path) -> eyre::Result<Self> {
        let metadata: Self = match serde_yaml::from_value(value.clone()) {
            Ok(config) => config,
            Err(err) => bail!(Error::InvalidMetadataFile {
                path: path.to_owned(),
                reason: err.to_string(),
            }),
        };

        Ok(metadata)
    }

    pub fn read_as_value(path: &Path) -> eyre::Result<serde_yaml::Value> {
        let metadata_file = File::open(path)?;

        let metadata: serde_yaml::Value = match serde_yaml::from_reader(metadata_file) {
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
    pub extra_values: YamlMapping,
}

impl EntryMetadata {
    // TODO this is ugly and unmaintainable; copy over via macro or
    // something.
    const FIELDS_TO_FILTER: &'static [&'static str] = &[
        "id",
        "title",
        "updated",
        "summary",
        "published",
        "author",
        "rights",
        "lang",
        "categories",
        "draft",
        "extra_variables",
    ];

    fn filter_extra_variables(values: YamlValue) -> YamlMapping {
        if let YamlValue::Mapping(mapping) = values {
            mapping
                .into_iter()
                .filter(|(key, _)| match key {
                    YamlValue::String(yaml_str) => {
                        !Self::FIELDS_TO_FILTER.contains(&yaml_str.as_str())
                    }
                    _ => true,
                })
                .collect::<YamlMapping>()
        } else {
            YamlMapping::new()
        }
    }

    pub fn read(path: &Path) -> eyre::Result<Self> {
        let raw_value = RawEntryMetadata::read_as_value(path).wrap_err(format!(
            "failed reading metadata file: {}",
            path.to_string_lossy()
        ))?;

        let raw = RawEntryMetadata::read_from_value(&raw_value, path).wrap_err(format!(
            "failed reading metadata file: {}",
            path.to_string_lossy()
        ))?;

        Ok(Self {
            id: raw.id,
            title: raw.title,
            extra_values: Self::filter_extra_variables(raw_value),
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

impl Entry {
    fn from_post_paths(
        path_pairs: &Vec<PathPair>,
        locator: impl Fn(PostLocationParams) -> eyre::Result<PostLocation>,
    ) -> eyre::Result<Vec<Self>> {
        let mut entries = Vec::new();

        // By this point, we've already removed post paths from the set that do not have an
        // accompanying metadata file.
        for PathPair {
            gemtext: gemtext_path,
            metadata: metadata_path,
        } in path_pairs
        {
            let post_body = String::from_utf8(
                fs::read(gemtext_path).wrap_err("failed reading gemtext post body")?,
            )
            .wrap_err("gemtext post body is not valid UTF-8")?;

            let post_metadata = EntryMetadata::read(metadata_path)?;

            // We do not publish draft posts.
            if post_metadata.draft {
                continue;
            }

            let post_slug = gemtext_path
                .file_stem()
                .ok_or(eyre!(
                    "This filename does not have a file stem. This is a bug.\n{}",
                    gemtext_path.to_string_lossy()
                ))?
                .to_string_lossy();

            let post_location = locator(PostLocationParams {
                metadata: &post_metadata,
                slug: &post_slug,
            })?;

            entries.push(Entry {
                metadata: post_metadata,
                body: post_body,
                url: post_location.url,
                path: post_location.path,
            });
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

        let path_pairs = check_mismatched_files(post_paths, &metadata_paths, warn_handler)
            .wrap_err("failed checking for mismatched post files")?;

        Self::from_post_paths(&path_pairs, locator)
    }
}
