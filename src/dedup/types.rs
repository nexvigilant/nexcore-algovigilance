//! Deduplication-specific types
//!
//! Tier: T3 (domain-specific PV deduplication types)
//! Grounds to: T1::Mapping (narrative → deduplicated set)

use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};

use crate::types::{CaseId, Similarity};

/// ICSR narrative for deduplication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IcsrNarrative {
    /// Case identifier
    pub case_id: CaseId,
    /// Full narrative text
    pub narrative_text: String,
    /// Report date
    pub report_date: Option<DateTime>,
    /// Drug names mentioned
    pub drug_names: Vec<String>,
    /// MedDRA event terms
    pub event_terms: Vec<String>,
}

/// A pair of cases compared for similarity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CasePair {
    /// First case
    pub case_a: CaseId,
    /// Second case
    pub case_b: CaseId,
    /// Similarity score
    pub similarity: Similarity,
    /// Whether this pair is a duplicate (above threshold)
    pub is_duplicate: bool,
}

/// Result of a deduplication run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeduplicationResult {
    /// Case IDs that are unique (not duplicates of another)
    pub unique_cases: Vec<CaseId>,
    /// All duplicate pairs found
    pub duplicate_pairs: Vec<CasePair>,
    /// Total input cases
    pub total_input: usize,
    /// Total unique cases
    pub total_unique: usize,
    /// Synonym pairs learned during this run
    pub synonym_pairs_learned: usize,
}

/// Configuration for deduplication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DedupConfig {
    /// Similarity threshold for duplicate detection (default: 0.85)
    pub similarity_threshold: f64,
    /// Whether to use learned synonyms from store
    pub use_learned_synonyms: bool,
    /// Maximum batch size for parallel comparison
    pub max_batch_size: usize,
}

impl Default for DedupConfig {
    fn default() -> Self {
        Self {
            similarity_threshold: 0.85,
            use_learned_synonyms: true,
            max_batch_size: 1000,
        }
    }
}

/// Synonym pair feedback for learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynonymPair {
    /// First term
    pub term_a: String,
    /// Equivalent term
    pub term_b: String,
}
