pub mod engine;
pub mod matcher;
pub mod query;
pub mod reference;
pub mod result;
pub mod walker;

pub use engine::{ParallelSearchEngine, SearchEngine};
pub use query::{
    CaseSensitivity, HiddenFilePolicy, MatchTarget, MountBoundaryPolicy, SearchQuery, SymlinkPolicy,
};
pub use reference::ReferenceSearchEngine;
pub use result::{FileTypeHint, NonFatalDiagnostic, SearchOutcome, SearchResult, normalize_paths};
pub use walker::device_is_within_boundary;
