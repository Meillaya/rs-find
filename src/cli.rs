use std::io::Write;
use std::path::PathBuf;

use crate::error::AppError;
use crate::search::{
    CaseSensitivity, HiddenFilePolicy, MatchTarget, MountBoundaryPolicy, ParallelSearchEngine,
    SearchEngine, SearchQuery, SymlinkPolicy,
};

const USAGE: &str = "Usage: rs-find [--path] [--ignore-case] [--exclude-hidden] [--cross-filesystems] <query> <root>\n\nFlags:\n  --path               Match against the full path instead of just the file name\n  --ignore-case        Match case-insensitively\n  --exclude-hidden     Skip hidden files and directories\n  --cross-filesystems  Traverse beyond the root filesystem boundary\n  -h, --help           Show this help text";

pub fn run<I, W, E>(args: I, stdout: &mut W, stderr: &mut E) -> u8
where
    I: IntoIterator<Item = String>,
    W: Write,
    E: Write,
{
    match parse_args(args) {
        Ok(Command::Help) => {
            let _ = writeln!(stdout, "{USAGE}");
            0
        }
        Ok(Command::Search(query)) => execute(query, stdout, stderr),
        Err(error) => {
            let _ = writeln!(stderr, "error: {error}\n\n{USAGE}");
            error.exit_code()
        }
    }
}

enum Command {
    Help,
    Search(SearchQuery),
}

fn parse_args<I>(args: I) -> Result<Command, AppError>
where
    I: IntoIterator<Item = String>,
{
    let mut args = args.into_iter();
    let _ = args.next();

    let mut match_target = MatchTarget::Name;
    let mut case_sensitivity = CaseSensitivity::Sensitive;
    let mut hidden_policy = HiddenFilePolicy::Include;
    let mut mount_boundary = MountBoundaryPolicy::StayOnRootFilesystem;
    let mut positional = Vec::new();

    for arg in args {
        match arg.as_str() {
            "-h" | "--help" => return Ok(Command::Help),
            "--path" => match_target = MatchTarget::Path,
            "--ignore-case" => case_sensitivity = CaseSensitivity::Insensitive,
            "--exclude-hidden" => hidden_policy = HiddenFilePolicy::Exclude,
            "--cross-filesystems" => mount_boundary = MountBoundaryPolicy::CrossFilesystems,
            _ if arg.starts_with('-') => {
                return Err(AppError::InvalidArguments(format!(
                    "unsupported flag: {arg}"
                )));
            }
            _ => positional.push(arg),
        }
    }

    if positional.len() != 2 {
        return Err(AppError::InvalidArguments(
            "expected <query> <root> positional arguments".to_owned(),
        ));
    }

    let root = PathBuf::from(&positional[1]);
    if !root.exists() {
        return Err(AppError::RootNotFound(root));
    }

    Ok(Command::Search(SearchQuery {
        root,
        pattern: positional.remove(0),
        match_target,
        case_sensitivity,
        hidden_policy,
        mount_boundary,
        symlink_policy: SymlinkPolicy::DoNotFollowDirectorySymlinks,
    }))
}

fn execute<W, E>(query: SearchQuery, stdout: &mut W, stderr: &mut E) -> u8
where
    W: Write,
    E: Write,
{
    let engine = ParallelSearchEngine::new();
    match engine.search(&query) {
        Ok(outcome) => {
            for result in outcome.results {
                let _ = writeln!(stdout, "{}", result.path.display());
            }
            for diagnostic in outcome.diagnostics {
                let _ = writeln!(
                    stderr,
                    "warning: {}: {}",
                    diagnostic.path.display(),
                    diagnostic.message
                );
            }
            0
        }
        Err(error) => {
            let _ = writeln!(stderr, "error: {error}");
            error.exit_code()
        }
    }
}
