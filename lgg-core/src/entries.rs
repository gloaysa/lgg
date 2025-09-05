use std::path::PathBuf;

/// Represents a non-critical issue that occurred during a query.
/// This is used to report problems (e.g., malformed files, invalid input)
/// without stopping a larger query operation.
#[derive(Debug)]
pub enum QueryError {
    InvalidDate { input: String, error: String },
    FileError { path: PathBuf, error: anyhow::Error },
}

/// The complete result of a query.
/// Contains successfully parsed tags and any errors.
#[derive(Debug)]
pub struct QueryTagsResult {
    pub tags: Vec<String>,
    pub errors: Vec<QueryError>,
}