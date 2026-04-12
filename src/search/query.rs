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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MountBoundaryPolicy {
    StayOnRootFilesystem,
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
