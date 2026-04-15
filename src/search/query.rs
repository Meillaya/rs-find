use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchTarget {
    Name,
    Path,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaseSensitivity {
    Sensitive,
    Insensitive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HiddenFilePolicy {
    Include,
    Exclude,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MountBoundaryPolicy {
    StayOnRootFilesystem,
    CrossFilesystems,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymlinkPolicy {
    DoNotFollowDirectorySymlinks,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchQuery {
    pub root: PathBuf,
    pub pattern: String,
    pub match_target: MatchTarget,
    pub case_sensitivity: CaseSensitivity,
    pub hidden_policy: HiddenFilePolicy,
    pub mount_boundary: MountBoundaryPolicy,
    pub symlink_policy: SymlinkPolicy,
}

pub fn path_is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.starts_with('.'))
}
