use std::fmt::{Display, Formatter};
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum AppError {
    InvalidArguments(String),
    RootNotFound(PathBuf),
    RootMetadata { path: PathBuf, source: io::Error },
}

impl AppError {
    pub fn exit_code(&self) -> u8 {
        match self {
            Self::InvalidArguments(_) => 2,
            Self::RootNotFound(_) | Self::RootMetadata { .. } => 1,
        }
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArguments(message) => write!(f, "{message}"),
            Self::RootNotFound(path) => write!(f, "search root does not exist: {}", path.display()),
            Self::RootMetadata { path, source } => {
                write!(
                    f,
                    "failed to read root metadata for {}: {source}",
                    path.display()
                )
            }
        }
    }
}

impl std::error::Error for AppError {}
