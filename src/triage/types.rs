//! Triage-specific types
//!
//! Tier: T3 (domain-specific signal triage types)
//! Grounds to: T1::Sequence (time-ordered decay)

use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};

use crate::types::{Relevance, SignalId};

/// A signal with decay-adjusted relevance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriagedSignal {
    /// Signal identifier (drug::event)
    pub signal_id: SignalId,
    /// Drug name
    pub drug: String,
    /// Event term
    pub event: String,
    /// PRR value from signal detection
    pub prr: f64,
    /// ROR value from signal detection
    pub ror: f64,
    /// Original confidence at first detection
    pub original_confidence: f64,
    /// Current decay-adjusted relevance
    pub current_relevance: Relevance,
    /// When last reinforced with new evidence
    pub last_reinforced: DateTime,
    /// When first detected
    pub first_detected: DateTime,
    /// Number of reinforcement events
    pub reinforcement_count: u32,
}

/// Configuration for triage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriageConfig {
    /// Half-life for decay in days (default: 30.0)
    pub half_life_days: f64,
    /// Minimum relevance before signal is dropped (default: 0.1)
    pub cutoff_relevance: f64,
    /// Maximum queue size (default: 1000)
    pub max_queue_size: usize,
}

impl Default for TriageConfig {
    fn default() -> Self {
        Self {
            half_life_days: 30.0,
            cutoff_relevance: 0.1,
            max_queue_size: 1000,
        }
    }
}

/// Result of a triage operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriageResult {
    /// Signals above cutoff
    pub active_signals: Vec<TriagedSignal>,
    /// Signals that fell below cutoff
    pub decayed_signals: Vec<TriagedSignal>,
    /// Total signals processed
    pub total: usize,
}

/// Input for creating a new signal to triage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalInput {
    /// Drug name
    pub drug: String,
    /// Event term
    pub event: String,
    /// PRR value
    pub prr: f64,
    /// ROR value
    pub ror: f64,
    /// Initial confidence (typically derived from signal strength)
    pub confidence: f64,
}

/// Reinforcement event feedback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReinforcementEvent {
    /// Signal to reinforce
    pub signal_id: SignalId,
    /// Number of new cases supporting this signal
    pub new_case_count: u32,
}
