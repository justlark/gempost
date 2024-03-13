use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use eyre::{eyre, WrapErr};
use url::Url;

use crate::entry::EntryMetadata;
use crate::entry_util::{check_mismatched_files, PathPair, METADATA_FILE_EXT, POST_FILE_EXT};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageEntry {
    pub metadata: EntryMetadata,
    pub variables: HashMap<String, serde_yaml::Value>,
    pub body: String,
    pub url: Url,
    pub path: PathBuf,
}

pub struct PageLocation {
    pub url: Url,
    pub path: PathBuf,
}

pub struct PageLocationParams<'a> {
    pub metadata: &'a EntryMetadata,
    pub slug: &'a str,
}

impl PageEntry {
    fn from_page_paths(
        path_pairs: &Vec<PathPair>,
        locator: impl Fn(PageLocationParams) -> eyre::Result<PageLocation>,
    ) -> eyre::Result<Vec<Self>> {
        let mut entries = Vec::new();

        // By this point, we've already removed page paths from the set that do not have an
        // accompanying metadata file.
        for PathPair {
            gemtext: gemtext_path,
            metadata: metadata_path,
        } in path_pairs
        {
            let page_body = String::from_utf8(
                fs::read(gemtext_path).wrap_err("failed reading gemtext post body")?,
            )
            .wrap_err("gemtext post body is not valid UTF-8")?;

            let standard_metadata = EntryMetadata::read(metadata_path)?;

            // We do not publish draft pages.
            if standard_metadata.draft {
                continue;
            }

            let post_slug = gemtext_path
                .file_stem()
                .ok_or(eyre!(
                    "This filename does not have a file stem. This is a bug.\n{}",
                    gemtext_path.to_string_lossy()
                ))?
                .to_string_lossy();

            let post_location = locator(PageLocationParams {
                metadata: &standard_metadata,
                slug: &post_slug,
            })?;

            entries.push(PageEntry {
                metadata: standard_metadata,
                variables: HashMap::new(),
                body: page_body,
                url: post_location.url,
                path: post_location.path,
            });
        }

        Ok(entries)
    }

    pub fn from_pages(
        pages_dir: &Path,
        locator: impl Fn(PageLocationParams) -> eyre::Result<PageLocation>,
        warn_handler: impl Fn(&str),
    ) -> eyre::Result<Vec<Self>> {
        let file_entries = fs::read_dir(pages_dir).wrap_err("failed reading pages directory")?;

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
                .wrap_err("failed reading pages directory")?
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

        Self::from_page_paths(&path_pairs, locator)
    }
}
