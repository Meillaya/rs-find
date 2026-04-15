mod support;

use std::path::PathBuf;

use rs_find::search::{
    CaseSensitivity, HiddenFilePolicy, MatchTarget, MountBoundaryPolicy, ParallelSearchEngine,
    ReferenceSearchEngine, SearchEngine, SearchQuery, SymlinkPolicy, normalize_paths,
};
use support::TestDir;

fn make_query(root: &std::path::Path, pattern: &str) -> SearchQuery {
    SearchQuery {
        root: root.to_path_buf(),
        pattern: pattern.to_owned(),
        match_target: MatchTarget::Name,
        case_sensitivity: CaseSensitivity::Sensitive,
        hidden_policy: HiddenFilePolicy::Include,
        mount_boundary: MountBoundaryPolicy::StayOnRootFilesystem,
        symlink_policy: SymlinkPolicy::DoNotFollowDirectorySymlinks,
    }
}

#[test]
fn recursive_search_finds_nested_matches() {
    let fixture = TestDir::new();
    fixture.create_file("alpha/target.txt", "hello");
    fixture.create_file("alpha/beta/target.log", "hello");
    fixture.create_file("alpha/beta/other.txt", "hello");

    let engine = ParallelSearchEngine::new();
    let outcome = engine
        .search(&make_query(fixture.path(), "target"))
        .expect("search should succeed");

    let normalized = normalize_paths(outcome.results.iter().map(|result| result.path.as_path()));
    assert_eq!(
        normalized,
        normalize_paths([
            fixture.path().join("alpha/target.txt"),
            fixture.path().join("alpha/beta/target.log"),
        ])
    );
}

#[test]
fn path_match_mode_checks_full_paths() {
    let fixture = TestDir::new();
    fixture.create_file("nested/example.txt", "hello");

    let mut query = make_query(fixture.path(), "nested/example");
    query.match_target = MatchTarget::Path;

    let engine = ParallelSearchEngine::new();
    let outcome = engine.search(&query).expect("search should succeed");
    assert_eq!(outcome.results.len(), 1);
}

#[test]
fn ignore_case_mode_matches_case_insensitively() {
    let fixture = TestDir::new();
    fixture.create_file("nested/Example.TXT", "hello");

    let mut query = make_query(fixture.path(), "example.txt");
    query.case_sensitivity = CaseSensitivity::Insensitive;

    let engine = ParallelSearchEngine::new();
    let outcome = engine.search(&query).expect("search should succeed");
    assert_eq!(outcome.results.len(), 1);
}

#[test]
fn non_matching_query_returns_no_results() {
    let fixture = TestDir::new();
    fixture.create_file("nested/Example.TXT", "hello");

    let engine = ParallelSearchEngine::new();
    let outcome = engine
        .search(&make_query(fixture.path(), "does-not-exist"))
        .expect("search should succeed");
    assert!(outcome.results.is_empty());
}

#[cfg(unix)]
#[test]
fn does_not_follow_directory_symlinks() {
    let fixture = TestDir::new();
    fixture.create_file("real-dir/hidden-target.txt", "hello");
    fixture.create_dir_symlink("real-dir", "linked-dir");

    let engine = ParallelSearchEngine::new();
    let outcome = engine
        .search(&make_query(fixture.path(), "hidden-target"))
        .expect("search should succeed");

    let normalized = normalize_paths(outcome.results.iter().map(|result| result.path.as_path()));
    assert_eq!(
        normalized,
        normalize_paths([fixture.path().join("real-dir/hidden-target.txt")])
    );
}

#[cfg(unix)]
#[test]
fn reports_matching_symlink_entries() {
    let fixture = TestDir::new();
    fixture.create_dir("real-dir");
    let symlink_path = fixture.create_dir_symlink("real-dir", "symlink-target-dir");

    let engine = ParallelSearchEngine::new();
    let outcome = engine
        .search(&make_query(fixture.path(), "symlink-target"))
        .expect("search should succeed");

    let normalized = normalize_paths(outcome.results.iter().map(|result| result.path.as_path()));
    assert_eq!(normalized, normalize_paths([symlink_path]));
}

#[cfg(unix)]
#[test]
fn permission_denied_is_non_fatal() {
    let fixture = TestDir::new();
    fixture.create_file("readable/target.txt", "hello");
    fixture.make_unreadable_dir("blocked");

    let engine = ParallelSearchEngine::new();
    let outcome = engine
        .search(&make_query(fixture.path(), "target"))
        .expect("search should succeed");

    let normalized = normalize_paths(outcome.results.iter().map(|result| result.path.as_path()));
    assert_eq!(
        normalized,
        normalize_paths([fixture.path().join("readable/target.txt")])
    );
    assert!(
        outcome
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.path.ends_with(PathBuf::from("blocked"))),
        "expected permission denied diagnostic",
    );
}

#[test]
fn hidden_files_are_included_by_default() {
    let fixture = TestDir::new();
    fixture.create_file(".secret-target", "hello");

    let engine = ParallelSearchEngine::new();
    let outcome = engine
        .search(&make_query(fixture.path(), "target"))
        .expect("search should succeed");

    assert_eq!(outcome.results.len(), 1);
}

#[test]
fn hidden_files_can_be_excluded_explicitly() {
    let fixture = TestDir::new();
    fixture.create_file(".secret-target", "hello");
    fixture.create_file("visible-target", "hello");

    let mut query = make_query(fixture.path(), "target");
    query.hidden_policy = HiddenFilePolicy::Exclude;

    let engine = ParallelSearchEngine::new();
    let outcome = engine.search(&query).expect("search should succeed");

    let normalized = normalize_paths(outcome.results.iter().map(|result| result.path.as_path()));
    assert_eq!(
        normalized,
        normalize_paths([fixture.path().join("visible-target")])
    );
}

#[test]
fn reference_and_parallel_engines_produce_equivalent_normalized_results() {
    let fixture = TestDir::new();
    fixture.create_file("one/target.txt", "hello");
    fixture.create_file("two/nested/target.md", "hello");
    fixture.create_file("two/nested/nope.md", "hello");

    let query = make_query(fixture.path(), "target");
    let parallel = ParallelSearchEngine::new()
        .search(&query)
        .expect("parallel search should succeed");
    let reference = ReferenceSearchEngine
        .search(&query)
        .expect("reference search should succeed");

    assert_eq!(
        normalize_paths(parallel.results.iter().map(|result| result.path.as_path())),
        normalize_paths(reference.results.iter().map(|result| result.path.as_path())),
    );
}

#[test]
fn reference_and_parallel_engines_remain_equivalent_under_explicit_policy_toggles() {
    let fixture = TestDir::new();
    fixture.create_file(".hidden-target", "hello");
    fixture.create_file("visible-target", "hello");

    let mut query = make_query(fixture.path(), "target");
    query.hidden_policy = HiddenFilePolicy::Exclude;
    query.mount_boundary = MountBoundaryPolicy::CrossFilesystems;

    let parallel = ParallelSearchEngine::new()
        .search(&query)
        .expect("parallel search should succeed");
    let reference = ReferenceSearchEngine
        .search(&query)
        .expect("reference search should succeed");

    assert_eq!(
        normalize_paths(parallel.results.iter().map(|result| result.path.as_path())),
        normalize_paths(reference.results.iter().map(|result| result.path.as_path())),
    );
}
