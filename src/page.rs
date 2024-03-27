use std::path::PathBuf;

use chrono::{DateTime, FixedOffset, Local};
use eyre::bail;
use url::Url;

use crate::config::{AuthorConfig, Config};
use crate::page_entry::{PageEntry, PageLocation, PageLocationParams};
use crate::template::{PagePathParams, PagePathTemplateData};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedAuthor {
    pub name: String,
    pub email: Option<String>,
    pub uri: Option<String>,
}

impl From<AuthorConfig> for FeedAuthor {
    fn from(value: AuthorConfig) -> Self {
        Self {
            name: value.name,
            email: value.email,
            uri: value.uri,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pages {
    pub capsule_url: Url,
    pub index_url: Url,
    pub updated: DateTime<FixedOffset>,
    pub rights: Option<String>,
    pub author: Option<FeedAuthor>,
    pub pages_path: PathBuf,
    pub pages: Vec<PageEntry>,
}

impl Pages {
    pub fn from_config(config: &Config, warn_handler: impl Fn(&str)) -> eyre::Result<Self> {
        let locator = |params: PageLocationParams| -> eyre::Result<PageLocation> {
            let mut page_url = config.url.clone();

            let path_params = PagePathTemplateData::from(PagePathParams {
                base_path: &config.pages_dir,
                file_path: &params.file_path,
                slug: params.slug,
            });

            let page_path = path_params.render(&config.page_path)?;

            let mut url_segments = match page_url.path_segments_mut() {
                Ok(segments) => segments,
                Err(()) => bail!("capsule URL cannot be a base URL"),
            };

            let mut page_filepath = PathBuf::new();

            for segment in page_path.split('/') {
                url_segments.push(segment);
                page_filepath.push(segment);
            }

            drop(url_segments);

            Ok(PageLocation {
                url: page_url,
                path: page_filepath,
            })
        };

        let entries = PageEntry::from_pages(&config.pages_dir, locator, warn_handler)?;

        // Get the time the most recently updated post was updated.
        let last_updated = entries
            .iter()
            .max_by_key(|entry| entry.metadata.updated)
            .map(|entry| entry.metadata.updated)
            .unwrap_or_else(|| Local::now().fixed_offset());

        let mut feed_url = config.url.clone();
        feed_url.set_path(&config.feed_path);

        let mut index_url = config.url.clone();
        index_url.set_path(&config.index_path);

        Ok(Pages {
            capsule_url: config.url.clone(),
            index_url,
            pages_path: config.pages_dir.clone(),
            pages: entries,
            updated: last_updated,
            rights: config.rights.clone(),
            author: config.author.as_ref().cloned().map(Into::into),
        })
    }
}
