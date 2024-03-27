use std::collections::HashSet;
use std::path::{Path, PathBuf};
use eyre::bail;

pub(crate) const POST_FILE_EXT: &str = "gmi";
pub(crate) const METADATA_FILE_EXT: &str = "yaml";

#[derive(Debug)]
pub struct PathPair {
    pub gemtext: PathBuf,
    pub metadata: PathBuf,
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
pub fn check_mismatched_files(
    post_paths: HashSet<PathBuf>,
    metadata_paths: &HashSet<PathBuf>,
    warn_handler: impl Fn(&str),
) -> eyre::Result<Vec<PathPair>> {
    // Warn about metadata files that don't have an accompanying gemtext file.
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
        }
    }

    let mut pairs = Vec::new();

    // Filter out gemtext files that don't have an accompanying metadata file.
    for post_path in post_paths.into_iter() {
        let maybe_metadata_path = match change_file_ext(&post_path, METADATA_FILE_EXT) {
            Some(path) => path,
            None => bail!("This file has no filename, even though we've already checked for one. This is a bug."),
        };

        if metadata_paths.contains(&maybe_metadata_path) {
            pairs.push(PathPair {
                gemtext: post_path,
                metadata: maybe_metadata_path,
            });
        } else {
            warn_handler(&format!(
                "This gemtext file does not have an accompanying YAML metadata file: {}",
                post_path.to_string_lossy()
            ));
        }
    }

    Ok(pairs)
}
