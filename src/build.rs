use std::path::PathBuf;

use crate::config::Config;
use crate::template::FeedTemplateData;

const FEED_TEMPLATE: &str = include_str!("atom.xml.tera");

pub fn build_site(config: &Config) -> eyre::Result<()> {
    let warn_handler = |msg: &str| eprintln!("{}", msg);

    let feed_data = FeedTemplateData::from_config(config, warn_handler)?;

    let index_page_path = config.public_dir.join(PathBuf::from_iter(
        config
            .index_path
            .split('/')
            .filter(|segment| !segment.is_empty()),
    ));

    feed_data.render_index(&config.index_template_file, &index_page_path)?;

    let feed_path = config.public_dir.join(PathBuf::from_iter(
        config
            .feed_path
            .split('/')
            .filter(|segment| !segment.is_empty()),
    ));

    feed_data.render_feed(FEED_TEMPLATE, &feed_path)?;

    // TODO: Render individual posts

    // TODO: Copy contents of ./static/

    Ok(())
}
