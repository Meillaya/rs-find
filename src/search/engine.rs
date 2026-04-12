use crate::error::AppError;

use super::query::SearchQuery;
use super::result::SearchOutcome;
use super::walker::ParallelWalker;

pub trait SearchEngine {
    fn search(&self, query: &SearchQuery) -> Result<SearchOutcome, AppError>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ParallelSearchEngine;

impl ParallelSearchEngine {
    pub const fn new() -> Self {
        Self
    }
}

impl SearchEngine for ParallelSearchEngine {
    fn search(&self, query: &SearchQuery) -> Result<SearchOutcome, AppError> {
        ParallelWalker::new().search(query)
    }
}
