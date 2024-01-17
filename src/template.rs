use std::collections::HashSet;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

use eyre::{bail, WrapErr};
use serde::{Deserialize, Serialize};
use tera::{Context, Tera};

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

const POST_FILE_EXT: &str = "gmi";
const METADATA_FILE_EXT: &str = "yaml";

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

impl EntryTemplateData {
    fn from_metadata(metadata: EntryMetadata, body: String, uri: String) -> Self {
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

    fn from_post_paths(post_paths: &HashSet<PathBuf>) -> eyre::Result<Vec<Self>> {
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

            // TODO: Implement post URL template parsing.
            entries.push(Self::from_metadata(
                post_metadata,
                post_body,
                String::from("TODO"),
            ));
        }

        Ok(entries)
    }

    pub fn from_posts(posts_dir: &Path, warn_handler: impl Fn(&str)) -> eyre::Result<Vec<Self>> {
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

        Self::from_post_paths(&post_paths)
    }

    pub fn render_post_page(&self, template: &Path, output: &Path) -> eyre::Result<()> {
        let mut tera = Tera::default();
        tera.add_template_file(template, Some("post"))
            .wrap_err("failed reading gemlog post page template")?;

        let mut context = Context::new();
        context.insert("entry", self);

        let dest_file = File::create(output).wrap_err("failed creating gemlog post page file")?;

        tera.render_to("post", &context, dest_file)
            .wrap_err("failed rendering gemlog post page from template")?;

        Ok(())
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

impl FeedTemplateData {
    pub fn render_index_page(&self, template: &Path, output: &Path) -> eyre::Result<()> {
        let mut tera = Tera::default();
        tera.add_template_file(template, Some("index"))
            .wrap_err("failed reading gemlog index page template")?;

        let mut context = Context::new();
        context.insert("feed", self);

        let dest_file = File::create(output).wrap_err("failed creating gemlog index page file")?;

        tera.render_to("index", &context, dest_file)
            .wrap_err("failed rendering gemlog index page from template")?;

        Ok(())
    }
}
