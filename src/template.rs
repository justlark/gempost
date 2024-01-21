use std::collections::HashSet;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Datelike, Local};
use eyre::{bail, eyre, WrapErr};
use serde::{Deserialize, Serialize};
use tera::{Context, Tera};
use url::Url;

use crate::config::Config;
use crate::error::Error;
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

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct EntryTemplateData {
    #[serde(skip)]
    pub slug: String,
    pub id: String,
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

struct Entry {
    metadata: EntryMetadata,
    body: String,
    url: Url,
    slug: String,
}

impl From<Entry> for EntryTemplateData {
    fn from(params: Entry) -> Self {
        Self {
            slug: params.slug,
            id: params.metadata.id,
            uri: params.url.to_string(),
            title: params.metadata.title,
            body: params.body,
            updated: params.metadata.updated,
            summary: params.metadata.summary,
            published: params.metadata.published,
            author: params.metadata.author.map(AuthorMetadata::into),
            rights: params.metadata.rights,
            lang: params.metadata.lang,
            categories: params.metadata.categories.unwrap_or_default(),
        }
    }
}

impl EntryTemplateData {
    fn from_post_paths(
        post_paths: &HashSet<PathBuf>,
        url_gen: impl Fn(&EntryMetadata, &str) -> eyre::Result<Url>,
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

            // If the `draft` property is missing, it's not a draft.
            if post_metadata.draft.unwrap_or(false) {
                continue;
            }

            let post_slug = post_path
                .file_stem()
                .ok_or(eyre!(
                    "This filename does not have a file stem. This is a bug.\n{}",
                    post_path.to_string_lossy()
                ))?
                .to_string_lossy();

            let post_url = url_gen(&post_metadata, &post_slug)?;

            entries.push(Self::from(Entry {
                metadata: post_metadata,
                body: post_body,
                url: post_url,
                slug: post_slug.into_owned(),
            }));
        }

        Ok(entries)
    }

    fn from_posts(
        posts_dir: &Path,
        url_gen: impl Fn(&EntryMetadata, &str) -> eyre::Result<Url>,
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

        Self::from_post_paths(&post_paths, url_gen)
    }

    pub fn render(&self, template: &Path, output: &Path) -> eyre::Result<()> {
        let mut tera = Tera::default();

        if let Err(err) = tera.add_template_file(template, Some("post")) {
            bail!(Error::InvalidPostPageTemplate {
                path: output.to_owned(),
                reason: err.to_string(),
            });
        }

        let mut context = Context::new();
        context.insert("entry", self);

        let dest_file = File::create(output).wrap_err("failed creating gemlog post page file")?;

        if let Err(err) = tera.render_to("post", &context, dest_file) {
            bail!(Error::InvalidPostPageTemplate {
                path: output.to_owned(),
                reason: err.to_string(),
            });
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct FeedTemplateData {
    pub id: String,
    pub feed_uri: String,
    pub index_uri: String,
    pub title: String,
    pub updated: String,
    pub subtitle: Option<String>,
    pub rights: Option<String>,
    pub author: Option<AuthorTemplateData>,
    pub entries: Vec<EntryTemplateData>,
}

impl FeedTemplateData {
    pub fn from_config(config: &Config, warn_handler: impl Fn(&str)) -> eyre::Result<Self> {
        let capsule_url = match Url::parse(&config.uri) {
            Ok(url) => url,
            Err(_) => bail!(Error::InvalidCapsuleUrl {
                url: config.uri.to_owned(),
            }),
        };

        let url_gen = |metadata: &EntryMetadata, slug: &str| -> eyre::Result<Url> {
            let mut path_url = capsule_url.clone();

            let path_params = PostPathTemplateData::from(PostPathParams {
                slug: slug.to_owned(),
                published: metadata
                    .published
                    .as_ref()
                    .map(|published| DateTime::parse_from_rfc3339(published).wrap_err(format!("Published date is not in RFC 3339 format, even though we've already checked. This is a bug.\n{}", published)))
                    .transpose()?,
            });

            let post_path = path_params.render(&config.post_path)?;

            let mut base_segments = match path_url.path_segments_mut() {
                Ok(segments) => segments,
                Err(()) => bail!("capsule base URI cannot be a base URL"),
            };

            for segment in post_path.split('/') {
                base_segments.push(segment);
            }

            drop(base_segments);

            Ok(path_url)
        };

        let entries = EntryTemplateData::from_posts(&config.posts_dir, url_gen, warn_handler)?;

        // Get the time the most recently updated post was updated.
        let last_updated = entries
            .iter()
            .max_by_key(|entry| &entry.updated)
            .map(|entry| entry.updated.clone())
            .unwrap_or(Local::now().to_rfc3339());

        let mut feed_url = capsule_url.clone();
        feed_url.set_path(&config.feed_path);

        let mut index_url = capsule_url.clone();
        index_url.set_path(&config.index_path);

        Ok(FeedTemplateData {
            id: config.uri.clone(),
            feed_uri: feed_url.to_string(),
            index_uri: index_url.to_string(),
            title: config.title.clone(),
            updated: last_updated,
            subtitle: config.subtitle.clone(),
            rights: config.rights.clone(),
            author: config
                .author
                .as_ref()
                .cloned()
                .map(AuthorTemplateData::from),
            entries,
        })
    }

    pub fn render_index(&self, template: &Path, output: &Path) -> eyre::Result<()> {
        let mut tera = Tera::default();

        if let Err(err) = tera.add_template_file(template, Some("index")) {
            bail!(Error::InvalidIndexPageTemplate {
                reason: err.to_string()
            });
        }

        let mut context = Context::new();
        context.insert("feed", self);

        let parent_dir = output.parent().ok_or(eyre!(
            "Could not get parent directory of index page file. This is a bug."
        ))?;

        fs::create_dir_all(parent_dir).wrap_err("failed creating parent directory")?;

        let dest_file = File::create(output).wrap_err("failed creating gemlog index page file")?;

        if let Err(err) = tera.render_to("index", &context, dest_file) {
            bail!(Error::InvalidIndexPageTemplate {
                reason: err.to_string(),
            });
        }

        Ok(())
    }

    pub fn render_feed(&self, template: &str, output: &Path) -> eyre::Result<()> {
        let mut tera = Tera::default();

        tera.add_raw_template("feed", template)
            .wrap_err("There was an issue generating the Atom feed. This is a bug.")?;

        let mut context = Context::new();
        context.insert("feed", self);

        let parent_dir = output.parent().ok_or(eyre!(
            "Could not get parent directory of Atom feed file. This is a bug."
        ))?;

        fs::create_dir_all(parent_dir).wrap_err("failed creating parent directory")?;

        let dest_file = File::create(output).wrap_err("failed creating gemlog Atom feed file")?;

        tera.render_to("feed", &context, dest_file)
            .wrap_err("failed generating the Atom feed")?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct PostPathParams {
    slug: String,
    published: Option<DateTime<chrono::FixedOffset>>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PostPathTemplateData {
    pub year: String,
    pub month: String,
    pub day: String,
    pub slug: String,
}

impl From<PostPathParams> for PostPathTemplateData {
    fn from(params: PostPathParams) -> Self {
        Self {
            year: params
                .published
                .map(|published| format!("{:0>4}", published.year()))
                .unwrap_or_default(),
            month: params
                .published
                .map(|published| format!("{:0>2}", published.month()))
                .unwrap_or_default(),
            day: params
                .published
                .map(|published| format!("{:0>2}", published.day()))
                .unwrap_or_default(),
            slug: params.slug,
        }
    }
}

impl PostPathTemplateData {
    pub fn render(&self, template: &str) -> eyre::Result<String> {
        let mut tera = Tera::default();

        if let Err(err) = tera.add_raw_template("path", template) {
            bail!(Error::InvalidPostPath {
                template: template.to_owned(),
                reason: err.to_string(),
            });
        }

        let mut context = Context::new();
        context.insert("year", &self.year);
        context.insert("month", &self.month);
        context.insert("day", &self.day);
        context.insert("slug", &self.slug);

        match tera.render("path", &context) {
            Ok(path) => Ok(path),
            Err(err) => bail!(Error::InvalidPostPath {
                template: template.to_owned(),
                reason: err.to_string(),
            }),
        }
    }
}
