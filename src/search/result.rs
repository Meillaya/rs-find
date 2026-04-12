use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileTypeHint {
    Directory,
    File,
    Symlink,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchResult {
    pub path: PathBuf,
    pub file_type: FileTypeHint,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonFatalDiagnostic {
    pub path: PathBuf,
    pub message: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SearchOutcome {
    pub results: Vec<SearchResult>,
    pub diagnostics: Vec<NonFatalDiagnostic>,
}

pub fn normalize_paths<I>(paths: I) -> Vec<String>
where
    I: IntoIterator,
    I::Item: AsRef<Path>,
{
    let mut normalized = paths
        .into_iter()
        .map(|path| path.as_ref().to_string_lossy().replace('\\', "/"))
        .collect::<Vec<_>>();
    normalized.sort();
    normalized
}
