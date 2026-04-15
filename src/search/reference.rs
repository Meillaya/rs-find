use std::fs;
use std::os::unix::fs::MetadataExt;

use crate::error::AppError;

use super::engine::SearchEngine;
use super::matcher;
use super::query::{HiddenFilePolicy, SearchQuery, SymlinkPolicy, path_is_hidden};
use super::result::{FileTypeHint, NonFatalDiagnostic, SearchOutcome, SearchResult};
use super::walker::device_is_within_boundary;

#[derive(Debug, Default, Clone, Copy)]
pub struct ReferenceSearchEngine;

impl SearchEngine for ReferenceSearchEngine {
    fn search(&self, query: &SearchQuery) -> Result<SearchOutcome, AppError> {
        let root_metadata = fs::metadata(&query.root).map_err(|source| AppError::RootMetadata {
            path: query.root.clone(),
            source,
        })?;
        let root_device = root_metadata.dev();
        let mut stack = vec![query.root.clone()];
        let mut outcome = SearchOutcome::default();

        while let Some(directory) = stack.pop() {
            let read_dir = match fs::read_dir(&directory) {
                Ok(read_dir) => read_dir,
                Err(error) => {
                    outcome.diagnostics.push(NonFatalDiagnostic {
                        path: directory.clone(),
                        message: error.to_string(),
                    });
                    continue;
                }
            };

            for entry in read_dir {
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(error) => {
                        outcome.diagnostics.push(NonFatalDiagnostic {
                            path: directory.clone(),
                            message: error.to_string(),
                        });
                        continue;
                    }
                };

                let path = entry.path();
                if matches!(query.hidden_policy, HiddenFilePolicy::Exclude) && path_is_hidden(&path)
                {
                    continue;
                }

                let symlink_metadata = match fs::symlink_metadata(&path) {
                    Ok(metadata) => metadata,
                    Err(error) => {
                        outcome.diagnostics.push(NonFatalDiagnostic {
                            path: path.clone(),
                            message: error.to_string(),
                        });
                        continue;
                    }
                };

                if matcher::matches(query, &path) {
                    outcome.results.push(SearchResult {
                        path: path.clone(),
                        file_type: file_type_hint(&symlink_metadata),
                    });
                }

                if symlink_metadata.file_type().is_symlink()
                    && matches!(
                        query.symlink_policy,
                        SymlinkPolicy::DoNotFollowDirectorySymlinks
                    )
                {
                    continue;
                }

                if symlink_metadata.is_dir() {
                    let metadata = match fs::metadata(&path) {
                        Ok(metadata) => metadata,
                        Err(error) => {
                            outcome.diagnostics.push(NonFatalDiagnostic {
                                path: path.clone(),
                                message: error.to_string(),
                            });
                            continue;
                        }
                    };

                    if device_is_within_boundary(query.mount_boundary, root_device, metadata.dev())
                    {
                        stack.push(path);
                    }
                }
            }
        }

        Ok(outcome)
    }
}

fn file_type_hint(metadata: &fs::Metadata) -> FileTypeHint {
    if metadata.file_type().is_symlink() {
        FileTypeHint::Symlink
    } else if metadata.is_dir() {
        FileTypeHint::Directory
    } else if metadata.is_file() {
        FileTypeHint::File
    } else {
        FileTypeHint::Other
    }
}
