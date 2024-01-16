use std::fs::File;
use std::path::Path;

use eyre::WrapErr;
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

impl EntryTemplateData {
    pub fn build_post_page(&self, template: &Path, output: &Path) -> eyre::Result<()> {
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

impl FeedTemplateData {
    pub fn build_index_page(&self, template: &Path, output: &Path) -> eyre::Result<()> {
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
