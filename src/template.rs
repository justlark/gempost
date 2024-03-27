use std::fs::{self, File};
use std::path::Path;

use chrono::{DateTime, Datelike, FixedOffset};
use eyre::{bail, eyre, WrapErr};
use serde::{Deserialize, Serialize};
use tera::{Context, Tera};

use crate::entry::{AuthorMetadata, Entry};
use crate::error::Error;
use crate::feed::{Feed, FeedAuthor};

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct EntryAuthorTemplateData {
    pub name: String,
    pub email: Option<String>,
    pub uri: Option<String>,
}

impl From<AuthorMetadata> for EntryAuthorTemplateData {
    fn from(value: AuthorMetadata) -> Self {
        Self {
            name: value.name,
            email: value.email,
            uri: value.uri,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct EntryTemplateData {
    pub id: String,
    pub url: String,
    pub title: String,
    pub body: String,
    pub updated: String,
    pub summary: Option<String>,
    pub published: Option<String>,
    pub author: Option<EntryAuthorTemplateData>,
    pub rights: Option<String>,
    pub lang: Option<String>,
    pub categories: Vec<String>,
}

impl From<Entry> for EntryTemplateData {
    fn from(params: Entry) -> Self {
        Self {
            id: params.metadata.id,
            url: params.url.to_string(),
            title: params.metadata.title,
            body: params.body,
            updated: params.metadata.updated.to_rfc3339(),
            summary: params.metadata.summary,
            published: params
                .metadata
                .published
                .as_ref()
                .map(DateTime::<FixedOffset>::to_rfc3339),
            author: params.metadata.author.map(Into::into),
            rights: params.metadata.rights,
            lang: params.metadata.lang,
            categories: params.metadata.categories,
        }
    }
}

impl EntryTemplateData {
    pub fn render(
        &self,
        feed: &FeedTemplateData,
        template: &Path,
        output: &Path,
    ) -> eyre::Result<()> {
        let mut tera = Tera::default();

        if let Err(err) = tera.add_template_file(template, Some("post")) {
            bail!(Error::InvalidPostPageTemplate {
                path: output.to_owned(),
                reason: err.to_string(),
            });
        }

        let mut context = Context::new();
        context.insert("entry", self);
        context.insert("feed", feed);

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

impl FeedTemplateData {
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

        // The template name needs the `.xml` extension to signal to Tera that all input should be
        // XML-escaped.
        tera.add_raw_template("feed.xml", template)
            .wrap_err("The bundled Atom feed template is invalid. This is a bug.")?;

        let mut context = Context::new();
        context.insert("feed", self);

        let parent_dir = output.parent().ok_or(eyre!(
            "Could not get parent directory of Atom feed file. This is a bug."
        ))?;

        fs::create_dir_all(parent_dir).wrap_err("failed creating parent directory")?;

        let dest_file = File::create(output).wrap_err("failed creating gemlog Atom feed file")?;

        tera.render_to("feed.xml", &context, dest_file)
            .wrap_err("failed generating the Atom feed")?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct PostPathParams {
    pub slug: String,
    pub published: Option<DateTime<chrono::FixedOffset>>,
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
            // If there is no publish date, these are empty strings.
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

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct FeedAuthorTemplateData {
    pub name: String,
    pub email: Option<String>,
    pub uri: Option<String>,
}

impl From<FeedAuthor> for FeedAuthorTemplateData {
    fn from(value: FeedAuthor) -> Self {
        Self {
            name: value.name,
            email: value.email,
            uri: value.uri,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct FeedTemplateData {
    pub capsule_url: String,
    pub feed_url: String,
    pub index_url: String,
    pub title: String,
    pub updated: String,
    pub subtitle: Option<String>,
    pub rights: Option<String>,
    pub author: Option<FeedAuthorTemplateData>,
    pub entries: Vec<EntryTemplateData>,
}

impl From<Feed> for FeedTemplateData {
    fn from(feed: Feed) -> Self {
        Self {
            capsule_url: feed.capsule_url.to_string(),
            feed_url: feed.feed_url.to_string(),
            index_url: feed.index_url.to_string(),
            title: feed.title,
            updated: feed.updated.to_rfc3339(),
            subtitle: feed.subtitle,
            rights: feed.rights,
            author: feed.author.map(Into::into),
            entries: feed.entries.into_iter().map(Into::into).collect(),
        }
    }
}
