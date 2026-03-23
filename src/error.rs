//! Error types for nexcore-algovigilance
//!
//! Unified error enum covering all function domains:
//! deduplication, triage, persistence, and integration failures.

/// Result alias for algovigilance operations
pub type Result<T> = std::result::Result<T, AlgovigilanceError>;

/// Errors from algovigilance functions
///
/// Tier: T2-C (cross-domain composite — aggregates IO, Brain, Vigilance errors)
/// Grounds to: T1::Recursion (enum variant dispatch)
#[derive(Debug, nexcore_error::Error)]
pub enum AlgovigilanceError {
    /// File system I/O failure
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Brain implicit knowledge store failure
    #[error("Brain error: {0}")]
    Brain(String),

    /// Vigilance computation failure
    #[error("Vigilance error: {0}")]
    Vigilance(String),

    /// Deduplication-specific failure
    #[error("Dedup error: {0}")]
    Dedup(String),

    /// Triage-specific failure
    #[error("Triage error: {0}")]
    Triage(String),

    /// Federated store failure
    #[error("Store error: {0}")]
    Store(String),

    /// Invalid input parameters
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// HTTP/network failure (FAERS integration)
    #[error("HTTP error: {0}")]
    Http(String),

    /// JSON serialization/deserialization failure
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
