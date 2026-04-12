use std::path::Path;

use super::query::{CaseSensitivity, MatchTarget, SearchQuery};

pub fn matches(query: &SearchQuery, path: &Path) -> bool {
    let candidate = match query.match_target {
        MatchTarget::Name => path
            .file_name()
            .map(|value| value.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string_lossy().into_owned()),
        MatchTarget::Path => path.to_string_lossy().into_owned(),
    };

    match query.case_sensitivity {
        CaseSensitivity::Sensitive => candidate.contains(&query.pattern),
        CaseSensitivity::Insensitive => candidate
            .to_lowercase()
            .contains(&query.pattern.to_lowercase()),
    }
}
