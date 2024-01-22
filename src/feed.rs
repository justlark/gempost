use std::cmp;
use std::path::PathBuf;

use chrono::{DateTime, FixedOffset, Local};
use eyre::bail;
use url::Url;

use crate::config::{AuthorConfig, Config};
use crate::entry::{Entry, PostLocation, PostLocationParams};
use crate::template::{PostPathParams, PostPathTemplateData};

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
pub struct Feed {
    pub capsule_url: Url,
    pub feed_url: Url,
    pub index_url: Url,
    pub title: String,
    pub updated: DateTime<FixedOffset>,
    pub subtitle: Option<String>,
    pub rights: Option<String>,
    pub author: Option<FeedAuthor>,
    pub entries: Vec<Entry>,
}

impl Feed {
    pub fn from_config(config: &Config, warn_handler: impl Fn(&str)) -> eyre::Result<Self> {
        let locator = |params: PostLocationParams| -> eyre::Result<PostLocation> {
            let mut post_url = config.url.clone();

            let path_params = PostPathTemplateData::from(PostPathParams {
                slug: params.slug.to_owned(),
                published: params.metadata.published,
            });

            let post_path = path_params.render(&config.post_path)?;

            let mut url_segments = match post_url.path_segments_mut() {
                Ok(segments) => segments,
                Err(()) => bail!("capsule URL cannot be a base URL"),
            };

            let mut post_filepath = PathBuf::new();

            for segment in post_path.split('/') {
                url_segments.push(segment);
                post_filepath.push(segment);
            }

            drop(url_segments);

            Ok(PostLocation {
                url: post_url,
                path: post_filepath,
            })
        };

        let mut entries = Entry::from_posts(&config.posts_dir, locator, warn_handler)?;

        // Sort entries in reverse-chronological order by publish time or, if there is no publish
        // time by last updated time.
        entries.sort_by_key(|entry| {
            cmp::Reverse(entry.metadata.published.unwrap_or(entry.metadata.updated))
        });

        // Get the time the most recently updated post was updated.
        let last_updated = entries
            .iter()
            .max_by_key(|entry| entry.metadata.updated)
            .map(|entry| entry.metadata.updated.clone())
            .unwrap_or_else(|| Local::now().fixed_offset());

        let mut feed_url = config.url.clone();
        feed_url.set_path(&config.feed_path);

        let mut index_url = config.url.clone();
        index_url.set_path(&config.index_path);

        Ok(Feed {
            capsule_url: config.url.clone(),
            feed_url,
            index_url,
            title: config.title.clone(),
            updated: last_updated,
            subtitle: config.subtitle.clone(),
            rights: config.rights.clone(),
            author: config.author.as_ref().cloned().map(Into::into),
            entries,
        })
    }
}
