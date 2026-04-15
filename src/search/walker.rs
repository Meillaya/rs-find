#[cfg(test)]
use std::collections::HashMap;
use std::collections::VecDeque;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

use crate::error::AppError;

use super::matcher;
use super::query::{
    HiddenFilePolicy, MountBoundaryPolicy, SearchQuery, SymlinkPolicy, path_is_hidden,
};
use super::result::{FileTypeHint, NonFatalDiagnostic, SearchOutcome, SearchResult};

#[derive(Debug, Default, Clone, Copy)]
pub struct ParallelWalker;

impl ParallelWalker {
    pub const fn new() -> Self {
        Self
    }

    pub fn search(&self, query: &SearchQuery) -> Result<SearchOutcome, AppError> {
        let root_metadata = fs::metadata(&query.root).map_err(|source| AppError::RootMetadata {
            path: query.root.clone(),
            source,
        })?;
        let root_device = root_metadata.dev();
        let worker_count = std::thread::available_parallelism()
            .map(std::num::NonZeroUsize::get)
            .unwrap_or(1);
        let queue = Arc::new(WorkQueue::new(query.root.clone()));
        let results = Arc::new(Mutex::new(Vec::<SearchResult>::new()));
        let diagnostics = Arc::new(Mutex::new(Vec::<NonFatalDiagnostic>::new()));
        let query = Arc::new(query.clone());
        let filesystem = RealFilesystem;

        thread::scope(|scope| {
            for _ in 0..worker_count {
                let queue = Arc::clone(&queue);
                let results = Arc::clone(&results);
                let diagnostics = Arc::clone(&diagnostics);
                let query = Arc::clone(&query);

                scope.spawn(move || {
                    while let Some(directory) = queue.pop() {
                        let scan = scan_directory(&filesystem, &directory, root_device, &query);
                        results
                            .lock()
                            .expect("results mutex poisoned")
                            .extend(scan.results);
                        diagnostics
                            .lock()
                            .expect("diagnostics mutex poisoned")
                            .extend(scan.diagnostics);
                        for child in scan.subdirectories {
                            queue.push(child);
                        }
                        queue.mark_complete();
                    }
                });
            }
        });

        let results = Arc::into_inner(results)
            .expect("results should have a single owner")
            .into_inner()
            .expect("results mutex should not be poisoned");
        let diagnostics = Arc::into_inner(diagnostics)
            .expect("diagnostics should have a single owner")
            .into_inner()
            .expect("diagnostics mutex should not be poisoned");

        Ok(SearchOutcome {
            results,
            diagnostics,
        })
    }
}

#[derive(Debug, Default)]
struct DirectoryScan {
    results: Vec<SearchResult>,
    subdirectories: Vec<PathBuf>,
    diagnostics: Vec<NonFatalDiagnostic>,
}

trait FilesystemView {
    fn read_dir_paths(&self, path: &Path) -> Result<Vec<PathBuf>, String>;
    fn symlink_metadata(&self, path: &Path) -> Result<MetadataSnapshot, String>;
    fn metadata(&self, path: &Path) -> Result<MetadataSnapshot, String>;
}

#[derive(Clone, Copy)]
struct RealFilesystem;

impl FilesystemView for RealFilesystem {
    fn read_dir_paths(&self, path: &Path) -> Result<Vec<PathBuf>, String> {
        let read_dir = fs::read_dir(path).map_err(|error| error.to_string())?;
        let mut entries = Vec::new();
        for entry in read_dir {
            let entry = entry.map_err(|error| error.to_string())?;
            entries.push(entry.path());
        }
        Ok(entries)
    }

    fn symlink_metadata(&self, path: &Path) -> Result<MetadataSnapshot, String> {
        let metadata = fs::symlink_metadata(path).map_err(|error| error.to_string())?;
        Ok(MetadataSnapshot::from_std(&metadata))
    }

    fn metadata(&self, path: &Path) -> Result<MetadataSnapshot, String> {
        let metadata = fs::metadata(path).map_err(|error| error.to_string())?;
        Ok(MetadataSnapshot::from_std(&metadata))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MetadataSnapshot {
    kind: EntryKind,
    device_id: u64,
}

impl MetadataSnapshot {
    fn from_std(metadata: &fs::Metadata) -> Self {
        let kind = if metadata.file_type().is_symlink() {
            EntryKind::Symlink
        } else if metadata.is_dir() {
            EntryKind::Directory
        } else if metadata.is_file() {
            EntryKind::File
        } else {
            EntryKind::Other
        };
        Self {
            kind,
            device_id: metadata.dev(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EntryKind {
    Directory,
    File,
    Symlink,
    Other,
}

fn scan_directory<F: FilesystemView>(
    filesystem: &F,
    directory: &Path,
    root_device: u64,
    query: &SearchQuery,
) -> DirectoryScan {
    let mut scan = DirectoryScan::default();
    let entries = match filesystem.read_dir_paths(directory) {
        Ok(entries) => entries,
        Err(message) => {
            scan.diagnostics.push(NonFatalDiagnostic {
                path: directory.to_path_buf(),
                message,
            });
            return scan;
        }
    };

    for path in entries {
        if matches!(query.hidden_policy, HiddenFilePolicy::Exclude) && path_is_hidden(&path) {
            continue;
        }

        let symlink_metadata = match filesystem.symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(message) => {
                scan.diagnostics.push(NonFatalDiagnostic {
                    path: path.clone(),
                    message,
                });
                continue;
            }
        };

        if matcher::matches(query, &path) {
            scan.results.push(SearchResult {
                path: path.clone(),
                file_type: file_type_hint(symlink_metadata.kind),
            });
        }

        if symlink_metadata.kind == EntryKind::Symlink
            && matches!(
                query.symlink_policy,
                SymlinkPolicy::DoNotFollowDirectorySymlinks
            )
        {
            continue;
        }

        if symlink_metadata.kind == EntryKind::Directory {
            let metadata = match filesystem.metadata(&path) {
                Ok(metadata) => metadata,
                Err(message) => {
                    scan.diagnostics.push(NonFatalDiagnostic {
                        path: path.clone(),
                        message,
                    });
                    continue;
                }
            };

            if device_is_within_boundary(query.mount_boundary, root_device, metadata.device_id) {
                scan.subdirectories.push(path);
            }
        }
    }

    scan
}

fn file_type_hint(kind: EntryKind) -> FileTypeHint {
    match kind {
        EntryKind::Directory => FileTypeHint::Directory,
        EntryKind::File => FileTypeHint::File,
        EntryKind::Symlink => FileTypeHint::Symlink,
        EntryKind::Other => FileTypeHint::Other,
    }
}

#[derive(Debug)]
struct QueueState {
    pending: VecDeque<PathBuf>,
    in_progress: usize,
    closed: bool,
}

#[derive(Debug)]
struct WorkQueue {
    state: Mutex<QueueState>,
    cvar: Condvar,
}

impl WorkQueue {
    fn new(root: PathBuf) -> Self {
        let mut pending = VecDeque::new();
        pending.push_back(root);
        Self {
            state: Mutex::new(QueueState {
                pending,
                in_progress: 0,
                closed: false,
            }),
            cvar: Condvar::new(),
        }
    }

    fn pop(&self) -> Option<PathBuf> {
        let mut state = self.state.lock().expect("queue mutex poisoned");
        loop {
            if let Some(path) = state.pending.pop_front() {
                state.in_progress += 1;
                return Some(path);
            }

            if state.closed {
                return None;
            }

            state = self
                .cvar
                .wait(state)
                .expect("queue mutex poisoned during wait");
        }
    }

    fn push(&self, path: PathBuf) {
        let mut state = self.state.lock().expect("queue mutex poisoned");
        state.pending.push_back(path);
        self.cvar.notify_one();
    }

    fn mark_complete(&self) {
        let mut state = self.state.lock().expect("queue mutex poisoned");
        state.in_progress = state.in_progress.saturating_sub(1);
        if state.in_progress == 0 && state.pending.is_empty() {
            state.closed = true;
            self.cvar.notify_all();
        }
    }
}

pub fn device_is_within_boundary(
    policy: MountBoundaryPolicy,
    root_device: u64,
    candidate_device: u64,
) -> bool {
    match policy {
        MountBoundaryPolicy::StayOnRootFilesystem => root_device == candidate_device,
        MountBoundaryPolicy::CrossFilesystems => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::normalize_paths;

    #[derive(Default)]
    struct FakeFilesystem {
        directories: HashMap<PathBuf, Vec<PathBuf>>,
        symlink_metadata: HashMap<PathBuf, MetadataSnapshot>,
        metadata: HashMap<PathBuf, MetadataSnapshot>,
    }

    impl FakeFilesystem {
        fn with_directory(mut self, directory: PathBuf, entries: Vec<PathBuf>) -> Self {
            self.directories.insert(directory, entries);
            self
        }

        fn with_symlink_metadata(mut self, path: PathBuf, metadata: MetadataSnapshot) -> Self {
            self.symlink_metadata.insert(path, metadata);
            self
        }

        fn with_metadata(mut self, path: PathBuf, metadata: MetadataSnapshot) -> Self {
            self.metadata.insert(path, metadata);
            self
        }
    }

    impl FilesystemView for FakeFilesystem {
        fn read_dir_paths(&self, path: &Path) -> Result<Vec<PathBuf>, String> {
            self.directories
                .get(path)
                .cloned()
                .ok_or_else(|| format!("missing fake directory: {}", path.display()))
        }

        fn symlink_metadata(&self, path: &Path) -> Result<MetadataSnapshot, String> {
            self.symlink_metadata
                .get(path)
                .copied()
                .ok_or_else(|| format!("missing fake symlink metadata: {}", path.display()))
        }

        fn metadata(&self, path: &Path) -> Result<MetadataSnapshot, String> {
            self.metadata
                .get(path)
                .copied()
                .ok_or_else(|| format!("missing fake metadata: {}", path.display()))
        }
    }

    fn make_query(root: &Path) -> SearchQuery {
        SearchQuery {
            root: root.to_path_buf(),
            pattern: "target".to_owned(),
            match_target: super::super::query::MatchTarget::Name,
            case_sensitivity: super::super::query::CaseSensitivity::Sensitive,
            hidden_policy: super::super::query::HiddenFilePolicy::Include,
            mount_boundary: super::super::query::MountBoundaryPolicy::StayOnRootFilesystem,
            symlink_policy: super::super::query::SymlinkPolicy::DoNotFollowDirectorySymlinks,
        }
    }

    #[test]
    fn device_boundary_rejects_other_devices() {
        assert!(device_is_within_boundary(
            super::super::query::MountBoundaryPolicy::StayOnRootFilesystem,
            7,
            7
        ));
        assert!(!device_is_within_boundary(
            super::super::query::MountBoundaryPolicy::StayOnRootFilesystem,
            7,
            8
        ));
    }

    #[test]
    fn cross_filesystem_policy_allows_other_devices() {
        assert!(device_is_within_boundary(
            super::super::query::MountBoundaryPolicy::CrossFilesystems,
            7,
            8
        ));
    }

    #[test]
    fn scan_directory_only_enqueues_same_device_directories() {
        let root = PathBuf::from("/virtual-root");
        let same_device = root.join("same-device-dir");
        let other_device = root.join("other-device-dir");
        let matching_file = root.join("target-file.txt");

        let fs = FakeFilesystem::default()
            .with_directory(
                root.clone(),
                vec![
                    same_device.clone(),
                    other_device.clone(),
                    matching_file.clone(),
                ],
            )
            .with_symlink_metadata(
                same_device.clone(),
                MetadataSnapshot {
                    kind: EntryKind::Directory,
                    device_id: 7,
                },
            )
            .with_symlink_metadata(
                other_device.clone(),
                MetadataSnapshot {
                    kind: EntryKind::Directory,
                    device_id: 8,
                },
            )
            .with_symlink_metadata(
                matching_file.clone(),
                MetadataSnapshot {
                    kind: EntryKind::File,
                    device_id: 7,
                },
            )
            .with_metadata(
                same_device.clone(),
                MetadataSnapshot {
                    kind: EntryKind::Directory,
                    device_id: 7,
                },
            )
            .with_metadata(
                other_device.clone(),
                MetadataSnapshot {
                    kind: EntryKind::Directory,
                    device_id: 8,
                },
            );

        let scan = scan_directory(&fs, &root, 7, &make_query(&root));
        assert_eq!(scan.subdirectories, vec![same_device]);
        assert_eq!(
            normalize_paths(scan.results.iter().map(|result| result.path.as_path())),
            normalize_paths([matching_file])
        );
    }

    #[test]
    fn scan_directory_skips_hidden_entries_when_policy_excludes_them() {
        let root = PathBuf::from("/virtual-root");
        let hidden_dir = root.join(".hidden-dir");
        let hidden_file = root.join(".target-file.txt");
        let visible_file = root.join("target-file.txt");

        let fs = FakeFilesystem::default()
            .with_directory(
                root.clone(),
                vec![
                    hidden_dir.clone(),
                    hidden_file.clone(),
                    visible_file.clone(),
                ],
            )
            .with_symlink_metadata(
                hidden_dir.clone(),
                MetadataSnapshot {
                    kind: EntryKind::Directory,
                    device_id: 7,
                },
            )
            .with_symlink_metadata(
                hidden_file.clone(),
                MetadataSnapshot {
                    kind: EntryKind::File,
                    device_id: 7,
                },
            )
            .with_symlink_metadata(
                visible_file.clone(),
                MetadataSnapshot {
                    kind: EntryKind::File,
                    device_id: 7,
                },
            );

        let mut query = make_query(&root);
        query.hidden_policy = super::super::query::HiddenFilePolicy::Exclude;

        let scan = scan_directory(&fs, &root, 7, &query);
        assert!(scan.subdirectories.is_empty());
        assert_eq!(
            normalize_paths(scan.results.iter().map(|result| result.path.as_path())),
            normalize_paths([visible_file])
        );
    }

    #[test]
    fn scan_directory_enqueues_other_devices_when_policy_allows_crossing() {
        let root = PathBuf::from("/virtual-root");
        let other_device = root.join("other-device-dir");

        let fs = FakeFilesystem::default()
            .with_directory(root.clone(), vec![other_device.clone()])
            .with_symlink_metadata(
                other_device.clone(),
                MetadataSnapshot {
                    kind: EntryKind::Directory,
                    device_id: 8,
                },
            )
            .with_metadata(
                other_device.clone(),
                MetadataSnapshot {
                    kind: EntryKind::Directory,
                    device_id: 8,
                },
            );

        let mut query = make_query(&root);
        query.mount_boundary = super::super::query::MountBoundaryPolicy::CrossFilesystems;

        let scan = scan_directory(&fs, &root, 7, &query);
        assert_eq!(scan.subdirectories, vec![other_device]);
    }
}
