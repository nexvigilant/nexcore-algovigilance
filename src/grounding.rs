//! # GroundsTo implementations for nexcore-algovigilance types
//!
//! Connects ICSR deduplication, signal triage, and federated persistence
//! types to the Lex Primitiva type system.
//!
//! ## Coverage
//!
//! - **types.rs**: Similarity, Relevance, HalfLife, CaseId, SignalId, DecayReport
//! - **error.rs**: AlgovigilanceError
//! - **store.rs**: SynonymEntry, AlgovigilanceStore
//! - **traits.rs**: (trait, not grounded — no concrete struct)
//! - **dedup/types.rs**: IcsrNarrative, CasePair, DeduplicationResult, DedupConfig, SynonymPair
//! - **dedup/mod.rs**: DedupFunction
//! - **triage/types.rs**: TriagedSignal, TriageConfig, TriageResult, SignalInput, ReinforcementEvent
//! - **triage/mod.rs**: TriageFunction
//! - **triage/classifier.rs**: UrgencyClassification, UrgencyClassifier
//! - **triage/queue.rs**: SignalQueue

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};
use nexcore_lex_primitiva::state_mode::StateMode;

// ============================================================================
// types.rs — shared newtypes
// ============================================================================

/// Similarity: T2-P (μ · N), dominant μ
///
/// Newtype wrapping f64 in [0.0, 1.0] representing narrative similarity.
/// Mapping-dominant: the score IS a transformation measure (how closely
/// one narrative maps onto another). Quantity provides the numeric magnitude.
impl GroundsTo for crate::types::Similarity {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // mu -- similarity is a mapping measure
            LexPrimitiva::Quantity, // N -- numeric magnitude [0.0, 1.0]
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

/// Relevance: T2-P (μ · N), dominant μ
///
/// Newtype wrapping f64 in [0.0, 1.0] representing decay-adjusted signal relevance.
/// Mapping-dominant: relevance scores the transformation from raw signal strength
/// through decay to current priority. Quantity provides the numeric magnitude.
impl GroundsTo for crate::types::Relevance {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // mu -- relevance maps signal strength through decay
            LexPrimitiva::Quantity, // N -- numeric magnitude [0.0, 1.0]
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

/// HalfLife: T2-P (pi · N), dominant pi
///
/// Newtype wrapping f64 representing exponential decay half-life in days.
/// Persistence-dominant: the entire purpose is to quantify how long information
/// persists before losing half its relevance. Quantity provides the day count.
impl GroundsTo for crate::types::HalfLife {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Persistence, // pi -- half-life governs persistence duration
            LexPrimitiva::Quantity,    // N -- numeric day count
        ])
        .with_dominant(LexPrimitiva::Persistence, 0.90)
    }
}

/// CaseId: T2-P (varsigma · sigma), dominant varsigma
///
/// Newtype wrapping String as an ICSR case identifier.
/// State-dominant: an identity token is encapsulated context. Each CaseId
/// names a unique point in the PV case space. Sequence provides the
/// string representation (ordered characters).
impl GroundsTo for crate::types::CaseId {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // varsigma -- identity token / encapsulated context
            LexPrimitiva::Sequence, // sigma -- string as ordered character sequence
        ])
        .with_dominant(LexPrimitiva::State, 0.90)
        .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

/// SignalId: T2-P (varsigma · sigma), dominant varsigma
///
/// Newtype wrapping String as a signal identifier (drug::event pair).
/// State-dominant: like CaseId, it is an identity token naming a specific
/// drug-event signal in the pharmacovigilance space.
impl GroundsTo for crate::types::SignalId {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // varsigma -- identity token for a signal
            LexPrimitiva::Sequence, // sigma -- string composition (drug::event)
        ])
        .with_dominant(LexPrimitiva::State, 0.90)
        .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

/// DecayReport: T2-C (sigma · N · emptyset · x), dominant sigma
///
/// Aggregated summary of a decay operation: items decayed, items below threshold,
/// min/max confidence. Sequence-dominant: the report summarizes the ordered
/// decay process. Quantity provides counts. Void covers Optional min/max.
/// Product covers the struct composition.
impl GroundsTo for crate::types::DecayReport {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence, // sigma -- ordered decay summary
            LexPrimitiva::Quantity, // N -- counts (items_decayed, items_below_threshold)
            LexPrimitiva::Void,     // emptyset -- Optional min/max confidence
            LexPrimitiva::Product,  // x -- struct combining multiple fields
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.80)
    }
}

// ============================================================================
// error.rs
// ============================================================================

/// AlgovigilanceError: T2-C (Sigma · partial · rho), dominant Sigma
///
/// Enum with 9 error variants covering IO, Brain, Vigilance, Dedup, Triage,
/// Store, InvalidInput, Http, and Json errors. Sum-dominant: the error IS a
/// discriminated union. Boundary provides error/success separation.
/// Recursion covers the enum dispatch pattern.
impl GroundsTo for crate::error::AlgovigilanceError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,       // Sigma -- enum variant dispatch
            LexPrimitiva::Boundary,  // partial -- error boundary
            LexPrimitiva::Recursion, // rho -- nested error conversion (#[from])
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

// ============================================================================
// store.rs
// ============================================================================

/// SynonymEntry: T2-C (mu · pi · N · x), dominant mu
///
/// A learned synonym pair with confidence and reinforcement count.
/// Mapping-dominant: the entry represents a bidirectional term-to-term mapping.
/// Persistence: learned and stored across sessions. Quantity: confidence score
/// and reinforcement count. Product: struct composition.
impl GroundsTo for crate::store::SynonymEntry {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,     // mu -- term_a <-> term_b mapping
            LexPrimitiva::Persistence, // pi -- stored across sessions
            LexPrimitiva::Quantity,    // N -- confidence + reinforcement_count
            LexPrimitiva::Product,     // x -- struct composition
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

/// AlgovigilanceStore: T3 (varsigma · pi · lambda · mu · sigma · partial), dominant pi
///
/// Federated persistence store with two backends (Brain implicit + dedicated disk).
/// Persistence-dominant: the store's entire purpose is durable state across sessions.
/// State: encapsulated in-memory state. Location: file paths for dedicated store.
/// Mapping: synonym key-value operations. Sequence: ordered synonym lists.
/// Boundary: error handling on IO operations.
impl GroundsTo for crate::store::AlgovigilanceStore {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Persistence, // pi -- durable storage across sessions
            LexPrimitiva::State,       // varsigma -- encapsulated DedicatedState
            LexPrimitiva::Location,    // lambda -- file paths (dedicated_path)
            LexPrimitiva::Mapping,     // mu -- synonym and queue key-value ops
            LexPrimitiva::Sequence,    // sigma -- ordered synonym lists
            LexPrimitiva::Boundary,    // partial -- IO error handling
        ])
        .with_dominant(LexPrimitiva::Persistence, 0.90)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

// ============================================================================
// dedup/types.rs
// ============================================================================

/// IcsrNarrative: T3 (varsigma · sigma · emptyset · x · mu · N), dominant varsigma
///
/// An ICSR case with narrative text, dates, drugs, and events for deduplication.
/// State-dominant: each narrative is a unique case snapshot with identity (case_id),
/// text content, optional date, and associated drug/event lists. Void covers the
/// Optional report_date. Product covers the struct composition.
impl GroundsTo for crate::dedup::types::IcsrNarrative {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // varsigma -- case snapshot with identity
            LexPrimitiva::Sequence, // sigma -- narrative text + drug/event lists
            LexPrimitiva::Void,     // emptyset -- Optional report_date
            LexPrimitiva::Product,  // x -- struct composition
            LexPrimitiva::Mapping,  // mu -- case_id maps to narrative content
            LexPrimitiva::Quantity, // N -- list lengths
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
        .with_state_mode(StateMode::Accumulated)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

/// CasePair: T2-C (kappa · mu · varsigma · x), dominant kappa
///
/// A pair of cases compared for similarity with a duplicate determination.
/// Comparison-dominant: the pair exists to answer the question "are these
/// two cases the same?" via similarity comparison and is_duplicate flag.
impl GroundsTo for crate::dedup::types::CasePair {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- similarity comparison / is_duplicate
            LexPrimitiva::Mapping,    // mu -- similarity score mapping
            LexPrimitiva::State,      // varsigma -- case_a, case_b identities
            LexPrimitiva::Product,    // x -- struct combining comparison results
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
        .with_state_mode(StateMode::Accumulated)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

/// DeduplicationResult: T3 (sigma · kappa · varsigma · N · mu · x), dominant sigma
///
/// Complete result of a batch deduplication run: unique cases, duplicate pairs,
/// input/output counts, synonyms learned. Sequence-dominant: the result
/// summarizes an ordered pipeline from input batch through pairwise comparison
/// to a deduplicated output set.
impl GroundsTo for crate::dedup::types::DeduplicationResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,   // sigma -- ordered pipeline result
            LexPrimitiva::Comparison, // kappa -- duplicate detection
            LexPrimitiva::State,      // varsigma -- unique_cases identities
            LexPrimitiva::Quantity,   // N -- total_input, total_unique, synonyms learned
            LexPrimitiva::Mapping,    // mu -- input set -> deduplicated set
            LexPrimitiva::Product,    // x -- struct composition
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.80)
        .with_state_mode(StateMode::Accumulated)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

/// DedupConfig: T2-C (partial · N · kappa · x), dominant partial
///
/// Configuration parameters for deduplication: similarity threshold,
/// synonym toggle, batch size. Boundary-dominant: the config defines the
/// thresholds and limits that bound dedup behavior.
impl GroundsTo for crate::dedup::types::DedupConfig {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // partial -- threshold/limit definitions
            LexPrimitiva::Quantity,   // N -- numeric thresholds and sizes
            LexPrimitiva::Comparison, // kappa -- similarity_threshold comparison
            LexPrimitiva::Product,    // x -- struct composition
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// SynonymPair: T2-P (mu · varsigma), dominant mu
///
/// Feedback type for learning: two equivalent terms (term_a, term_b).
/// Mapping-dominant: the pair IS a bidirectional mapping between terms.
/// State provides the identity context of each term.
impl GroundsTo for crate::dedup::types::SynonymPair {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping, // mu -- term_a <-> term_b equivalence
            LexPrimitiva::State,   // varsigma -- term identity context
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.90)
        .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

/// DedupFunction: T3 (mu · varsigma · kappa · pi · sigma · arrow), dominant mu
///
/// ICSR deduplication function with learned synonym boosting.
/// Mapping-dominant: the function transforms a narrative set into a
/// deduplicated set (input_narratives -> unique + duplicates). State:
/// internal synonyms list. Comparison: pairwise similarity testing.
/// Persistence: synonym learning across invocations. Sequence: batch
/// processing order. Causality: process/learn/decay lifecycle.
impl GroundsTo for crate::dedup::DedupFunction {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,     // mu -- narratives -> deduplicated set
            LexPrimitiva::State,       // varsigma -- internal synonym state
            LexPrimitiva::Comparison,  // kappa -- pairwise similarity testing
            LexPrimitiva::Persistence, // pi -- synonym learning across calls
            LexPrimitiva::Sequence,    // sigma -- batch processing order
            LexPrimitiva::Causality,   // arrow -- process/learn/decay lifecycle
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

// ============================================================================
// triage/types.rs
// ============================================================================

/// TriagedSignal: T3 (varsigma · N · mu · pi · nu · sigma · x), dominant varsigma
///
/// A signal with decay-adjusted relevance, detection timestamps,
/// reinforcement history, and PRR/ROR metrics. State-dominant: each
/// triaged signal is a rich case snapshot with mutable relevance.
/// Quantity: PRR, ROR, confidence, reinforcement count. Mapping:
/// original -> current relevance transformation. Persistence: timestamps
/// for temporal tracking. Frequency: reinforcement count rate.
/// Sequence: temporal ordering. Product: struct composition.
impl GroundsTo for crate::triage::types::TriagedSignal {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,       // varsigma -- signal snapshot with mutable relevance
            LexPrimitiva::Quantity,    // N -- PRR, ROR, confidence, reinforcement_count
            LexPrimitiva::Mapping,     // mu -- original -> current relevance decay
            LexPrimitiva::Persistence, // pi -- first_detected, last_reinforced timestamps
            LexPrimitiva::Frequency,   // nu -- reinforcement rate
            LexPrimitiva::Sequence,    // sigma -- temporal ordering
            LexPrimitiva::Product,     // x -- struct composition
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

/// TriageConfig: T2-C (partial · N · pi · x), dominant partial
///
/// Configuration for triage operations: half-life, cutoff relevance,
/// max queue size. Boundary-dominant: the config defines limits and
/// thresholds that bound triage behavior.
impl GroundsTo for crate::triage::types::TriageConfig {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,    // partial -- threshold/limit definitions
            LexPrimitiva::Quantity,    // N -- numeric thresholds (half_life, cutoff, max_size)
            LexPrimitiva::Persistence, // pi -- half-life governs decay persistence
            LexPrimitiva::Product,     // x -- struct composition
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// TriageResult: T3 (sigma · kappa · varsigma · N · mu · x), dominant sigma
///
/// Result of a triage operation: active signals, decayed signals, total count.
/// Sequence-dominant: the result partitions an ordered signal queue into
/// active and decayed subsets based on relevance cutoff.
impl GroundsTo for crate::triage::types::TriageResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,   // sigma -- ordered result partitioning
            LexPrimitiva::Comparison, // kappa -- active vs decayed partitioning
            LexPrimitiva::State,      // varsigma -- signal identities
            LexPrimitiva::Quantity,   // N -- total count
            LexPrimitiva::Mapping,    // mu -- queue -> partitioned result
            LexPrimitiva::Product,    // x -- struct composition
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.80)
        .with_state_mode(StateMode::Accumulated)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

/// SignalInput: T2-C (x · N · varsigma · mu), dominant x
///
/// Input for creating a new signal to triage: drug, event, PRR, ROR, confidence.
/// Product-dominant: a flat record combining named fields with no behavior.
/// Quantity: PRR, ROR, confidence. State: drug and event names.
/// Mapping: transforms into a TriagedSignal on insertion.
impl GroundsTo for crate::triage::types::SignalInput {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Product,  // x -- flat record type
            LexPrimitiva::Quantity, // N -- PRR, ROR, confidence
            LexPrimitiva::State,    // varsigma -- drug and event identity
            LexPrimitiva::Mapping,  // mu -- transforms into TriagedSignal
        ])
        .with_dominant(LexPrimitiva::Product, 0.85)
        .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

/// ReinforcementEvent: T2-P (arrow · varsigma · N), dominant arrow
///
/// Feedback event that reinforces a signal with new evidence.
/// Causality-dominant: the event IS a cause (new cases) producing
/// an effect (boosted relevance). State: signal_id identity.
/// Quantity: new_case_count.
impl GroundsTo for crate::triage::types::ReinforcementEvent {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality, // arrow -- cause (new cases) -> effect (boost)
            LexPrimitiva::State,     // varsigma -- signal_id target
            LexPrimitiva::Quantity,  // N -- new_case_count
        ])
        .with_dominant(LexPrimitiva::Causality, 0.85)
        .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

/// TriageFunction: T3 (sigma · varsigma · pi · arrow · mu · partial), dominant sigma
///
/// Signal triage function managing a priority queue with decay and reinforcement.
/// Sequence-dominant: the function manages a time-ordered priority queue where
/// signals are processed in relevance order. State: internal SignalQueue.
/// Persistence: signals persist and decay over time. Causality: process/learn/decay
/// lifecycle. Mapping: signal input -> triage result. Boundary: cutoff thresholds.
impl GroundsTo for crate::triage::TriageFunction {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,    // sigma -- time-ordered priority queue
            LexPrimitiva::State,       // varsigma -- internal SignalQueue state
            LexPrimitiva::Persistence, // pi -- signals decay over time
            LexPrimitiva::Causality,   // arrow -- process/learn/decay lifecycle
            LexPrimitiva::Mapping,     // mu -- signal input -> triage result
            LexPrimitiva::Boundary,    // partial -- cutoff thresholds
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.85)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

// ============================================================================
// triage/classifier.rs
// ============================================================================

/// UrgencyClassification: T2-C (kappa · mu · sigma · N), dominant kappa
///
/// Result of classifying a signal's urgency: urgency label, confidence,
/// decision path. Comparison-dominant: the classification IS a predicate
/// result (which urgency category does this signal match?).
impl GroundsTo for crate::triage::classifier::UrgencyClassification {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- urgency category matching
            LexPrimitiva::Mapping,    // mu -- features -> urgency label
            LexPrimitiva::Sequence,   // sigma -- decision path steps
            LexPrimitiva::Quantity,   // N -- confidence score
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

/// UrgencyClassifier: T3 (kappa · mu · varsigma · sigma · arrow · N), dominant kappa
///
/// Decision tree-backed signal urgency classifier. Trains from historical
/// data and classifies signals into urgency levels.
/// Comparison-dominant: the classifier's core operation is matching signal
/// features against trained decision boundaries. Mapping: features -> label.
/// State: trained tree state. Sequence: decision path traversal.
/// Causality: training causes classification capability. Quantity: feature values.
impl GroundsTo for crate::triage::classifier::UrgencyClassifier {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // kappa -- decision boundary matching
            LexPrimitiva::Mapping,    // mu -- features -> urgency label
            LexPrimitiva::State,      // varsigma -- trained tree state
            LexPrimitiva::Sequence,   // sigma -- decision path traversal
            LexPrimitiva::Causality,  // arrow -- training -> classification
            LexPrimitiva::Quantity,   // N -- feature values and confidence
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

// ============================================================================
// triage/queue.rs
// ============================================================================

/// SignalQueue: T3 (sigma · varsigma · kappa · pi · mu · partial), dominant sigma
///
/// Priority queue maintaining signals sorted by relevance with insert, decay,
/// reinforce, and find operations. Sequence-dominant: the queue IS an ordered
/// collection sorted by relevance. State: encapsulated signal list, half-life,
/// cutoff. Comparison: sorting by relevance. Persistence: decay over time.
/// Mapping: insert transforms input to signal. Boundary: cutoff and max_size limits.
impl GroundsTo for crate::triage::queue::SignalQueue {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,    // sigma -- ordered by relevance
            LexPrimitiva::State,       // varsigma -- encapsulated signal list
            LexPrimitiva::Comparison,  // kappa -- sorting by relevance
            LexPrimitiva::Persistence, // pi -- decay over time
            LexPrimitiva::Mapping,     // mu -- input -> signal transformation
            LexPrimitiva::Boundary,    // partial -- cutoff + max_size limits
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.85)
        .with_state_mode(StateMode::Mutable)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    // ---- T2-P types (2-3 unique primitives) ----

    #[test]
    fn similarity_grounds_to_mapping() {
        assert_eq!(
            crate::types::Similarity::dominant_primitive(),
            Some(LexPrimitiva::Mapping)
        );
        assert_eq!(crate::types::Similarity::tier(), Tier::T2Primitive);
    }

    #[test]
    fn relevance_grounds_to_mapping() {
        assert_eq!(
            crate::types::Relevance::dominant_primitive(),
            Some(LexPrimitiva::Mapping)
        );
        assert_eq!(crate::types::Relevance::tier(), Tier::T2Primitive);
    }

    #[test]
    fn half_life_grounds_to_persistence() {
        assert_eq!(
            crate::types::HalfLife::dominant_primitive(),
            Some(LexPrimitiva::Persistence)
        );
        assert_eq!(crate::types::HalfLife::tier(), Tier::T2Primitive);
    }

    #[test]
    fn case_id_grounds_to_state() {
        assert_eq!(
            crate::types::CaseId::dominant_primitive(),
            Some(LexPrimitiva::State)
        );
        assert_eq!(crate::types::CaseId::tier(), Tier::T2Primitive);
    }

    #[test]
    fn signal_id_grounds_to_state() {
        assert_eq!(
            crate::types::SignalId::dominant_primitive(),
            Some(LexPrimitiva::State)
        );
        assert_eq!(crate::types::SignalId::tier(), Tier::T2Primitive);
    }

    #[test]
    fn synonym_pair_grounds_to_mapping() {
        assert_eq!(
            crate::dedup::types::SynonymPair::dominant_primitive(),
            Some(LexPrimitiva::Mapping)
        );
        assert_eq!(crate::dedup::types::SynonymPair::tier(), Tier::T2Primitive);
    }

    #[test]
    fn reinforcement_event_grounds_to_causality() {
        assert_eq!(
            crate::triage::types::ReinforcementEvent::dominant_primitive(),
            Some(LexPrimitiva::Causality)
        );
        assert_eq!(
            crate::triage::types::ReinforcementEvent::tier(),
            Tier::T2Primitive
        );
    }

    // ---- T2-C types (4-5 unique primitives) ----

    #[test]
    fn decay_report_grounds_to_sequence() {
        assert_eq!(
            crate::types::DecayReport::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
        assert_eq!(crate::types::DecayReport::tier(), Tier::T2Composite);
    }

    #[test]
    fn algo_error_grounds_to_sum() {
        assert_eq!(
            crate::error::AlgovigilanceError::dominant_primitive(),
            Some(LexPrimitiva::Sum)
        );
        assert_eq!(crate::error::AlgovigilanceError::tier(), Tier::T2Primitive);
    }

    #[test]
    fn synonym_entry_grounds_to_mapping() {
        assert_eq!(
            crate::store::SynonymEntry::dominant_primitive(),
            Some(LexPrimitiva::Mapping)
        );
        assert_eq!(crate::store::SynonymEntry::tier(), Tier::T2Composite);
    }

    #[test]
    fn case_pair_grounds_to_comparison() {
        assert_eq!(
            crate::dedup::types::CasePair::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
        assert_eq!(crate::dedup::types::CasePair::tier(), Tier::T2Composite);
    }

    #[test]
    fn dedup_config_grounds_to_boundary() {
        assert_eq!(
            crate::dedup::types::DedupConfig::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
        assert_eq!(crate::dedup::types::DedupConfig::tier(), Tier::T2Composite);
    }

    #[test]
    fn triage_config_grounds_to_boundary() {
        assert_eq!(
            crate::triage::types::TriageConfig::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
        assert_eq!(
            crate::triage::types::TriageConfig::tier(),
            Tier::T2Composite
        );
    }

    #[test]
    fn signal_input_grounds_to_product() {
        assert_eq!(
            crate::triage::types::SignalInput::dominant_primitive(),
            Some(LexPrimitiva::Product)
        );
        assert_eq!(crate::triage::types::SignalInput::tier(), Tier::T2Composite);
    }

    #[test]
    fn urgency_classification_grounds_to_comparison() {
        assert_eq!(
            crate::triage::classifier::UrgencyClassification::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
        assert_eq!(
            crate::triage::classifier::UrgencyClassification::tier(),
            Tier::T2Composite
        );
    }

    // ---- T3 types (6+ unique primitives) ----

    #[test]
    fn icsr_narrative_is_t3() {
        assert_eq!(
            crate::dedup::types::IcsrNarrative::dominant_primitive(),
            Some(LexPrimitiva::State)
        );
        assert_eq!(
            crate::dedup::types::IcsrNarrative::tier(),
            Tier::T3DomainSpecific
        );
    }

    #[test]
    fn dedup_result_is_t3() {
        assert_eq!(
            crate::dedup::types::DeduplicationResult::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
        assert_eq!(
            crate::dedup::types::DeduplicationResult::tier(),
            Tier::T3DomainSpecific
        );
    }

    #[test]
    fn dedup_function_is_t3() {
        assert_eq!(
            crate::dedup::DedupFunction::dominant_primitive(),
            Some(LexPrimitiva::Mapping)
        );
        assert_eq!(crate::dedup::DedupFunction::tier(), Tier::T3DomainSpecific);
    }

    #[test]
    fn algo_store_is_t3() {
        assert_eq!(
            crate::store::AlgovigilanceStore::dominant_primitive(),
            Some(LexPrimitiva::Persistence)
        );
        assert_eq!(
            crate::store::AlgovigilanceStore::tier(),
            Tier::T3DomainSpecific
        );
    }

    #[test]
    fn triaged_signal_is_t3() {
        assert_eq!(
            crate::triage::types::TriagedSignal::dominant_primitive(),
            Some(LexPrimitiva::State)
        );
        assert_eq!(
            crate::triage::types::TriagedSignal::tier(),
            Tier::T3DomainSpecific
        );
    }

    #[test]
    fn triage_result_is_t3() {
        assert_eq!(
            crate::triage::types::TriageResult::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
        assert_eq!(
            crate::triage::types::TriageResult::tier(),
            Tier::T3DomainSpecific
        );
    }

    #[test]
    fn triage_function_is_t3() {
        assert_eq!(
            crate::triage::TriageFunction::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
        assert_eq!(
            crate::triage::TriageFunction::tier(),
            Tier::T3DomainSpecific
        );
    }

    #[test]
    fn urgency_classifier_is_t3() {
        assert_eq!(
            crate::triage::classifier::UrgencyClassifier::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
        assert_eq!(
            crate::triage::classifier::UrgencyClassifier::tier(),
            Tier::T3DomainSpecific
        );
    }

    #[test]
    fn signal_queue_is_t3() {
        assert_eq!(
            crate::triage::queue::SignalQueue::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
        assert_eq!(
            crate::triage::queue::SignalQueue::tier(),
            Tier::T3DomainSpecific
        );
    }

    // ---- Confidence range validation ----

    #[test]
    fn all_confidences_in_valid_range() {
        let types_and_compositions: Vec<(&str, PrimitiveComposition)> = vec![
            (
                "Similarity",
                crate::types::Similarity::primitive_composition(),
            ),
            (
                "Relevance",
                crate::types::Relevance::primitive_composition(),
            ),
            ("HalfLife", crate::types::HalfLife::primitive_composition()),
            ("CaseId", crate::types::CaseId::primitive_composition()),
            ("SignalId", crate::types::SignalId::primitive_composition()),
            (
                "DecayReport",
                crate::types::DecayReport::primitive_composition(),
            ),
            (
                "AlgovigilanceError",
                crate::error::AlgovigilanceError::primitive_composition(),
            ),
            (
                "SynonymEntry",
                crate::store::SynonymEntry::primitive_composition(),
            ),
            (
                "AlgovigilanceStore",
                crate::store::AlgovigilanceStore::primitive_composition(),
            ),
            (
                "IcsrNarrative",
                crate::dedup::types::IcsrNarrative::primitive_composition(),
            ),
            (
                "CasePair",
                crate::dedup::types::CasePair::primitive_composition(),
            ),
            (
                "DeduplicationResult",
                crate::dedup::types::DeduplicationResult::primitive_composition(),
            ),
            (
                "DedupConfig",
                crate::dedup::types::DedupConfig::primitive_composition(),
            ),
            (
                "SynonymPair",
                crate::dedup::types::SynonymPair::primitive_composition(),
            ),
            (
                "DedupFunction",
                crate::dedup::DedupFunction::primitive_composition(),
            ),
            (
                "TriagedSignal",
                crate::triage::types::TriagedSignal::primitive_composition(),
            ),
            (
                "TriageConfig",
                crate::triage::types::TriageConfig::primitive_composition(),
            ),
            (
                "TriageResult",
                crate::triage::types::TriageResult::primitive_composition(),
            ),
            (
                "SignalInput",
                crate::triage::types::SignalInput::primitive_composition(),
            ),
            (
                "ReinforcementEvent",
                crate::triage::types::ReinforcementEvent::primitive_composition(),
            ),
            (
                "TriageFunction",
                crate::triage::TriageFunction::primitive_composition(),
            ),
            (
                "UrgencyClassification",
                crate::triage::classifier::UrgencyClassification::primitive_composition(),
            ),
            (
                "UrgencyClassifier",
                crate::triage::classifier::UrgencyClassifier::primitive_composition(),
            ),
            (
                "SignalQueue",
                crate::triage::queue::SignalQueue::primitive_composition(),
            ),
        ];

        for (name, comp) in &types_and_compositions {
            assert!(
                comp.confidence >= 0.80 && comp.confidence <= 0.95,
                "{name} confidence {conf} outside [0.80, 0.95]",
                conf = comp.confidence
            );
            assert!(comp.dominant.is_some(), "{name} has no dominant primitive");
            assert!(
                !comp.primitives.is_empty(),
                "{name} has empty primitive list"
            );
        }
    }
}
