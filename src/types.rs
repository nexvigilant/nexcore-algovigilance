//! Shared newtypes for algovigilance functions
//!
//! Commandment IV (WRAP): No naked primitives for domain values.
//! All quantities are clamped to valid ranges on construction.

use serde::{Deserialize, Serialize};

/// Narrative similarity score in [0.0, 1.0]
///
/// Tier: T2-P (cross-domain primitive — wraps f64)
/// Grounds to T1 Mapping (μ) and Quantity (N): similarity magnitude
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Similarity(f64);

impl Similarity {
    /// Create a new similarity, clamped to [0.0, 1.0]
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    /// Get the inner value
    #[must_use]
    pub fn value(self) -> f64 {
        self.0
    }
}

impl From<f64> for Similarity {
    fn from(v: f64) -> Self {
        Self::new(v)
    }
}

/// Signal relevance score in [0.0, 1.0]
///
/// Tier: T2-P (cross-domain primitive — wraps f64)
/// Grounds to T1 Mapping (μ) and Quantity (N): relevance magnitude
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Relevance(f64);

impl Relevance {
    /// Create a new relevance, clamped to [0.0, 1.0]
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    /// Get the inner value
    #[must_use]
    pub fn value(self) -> f64 {
        self.0
    }
}

impl From<f64> for Relevance {
    fn from(v: f64) -> Self {
        Self::new(v)
    }
}

/// Exponential decay half-life in days (positive)
///
/// Tier: T2-P (cross-domain primitive — wraps f64)
/// Grounds to T1 Persistence (π) and Quantity (N): decay half-life
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HalfLife(f64);

impl HalfLife {
    /// Default half-life: 30 days
    pub const DEFAULT_DAYS: f64 = 30.0;

    /// Create a new half-life, must be positive (defaults to 30.0 if not)
    #[must_use]
    pub fn new(days: f64) -> Self {
        if days > 0.0 {
            Self(days)
        } else {
            Self(Self::DEFAULT_DAYS)
        }
    }

    /// Get the inner value in days
    #[must_use]
    pub fn days(self) -> f64 {
        self.0
    }
}

impl Default for HalfLife {
    fn default() -> Self {
        Self(Self::DEFAULT_DAYS)
    }
}

/// ICSR case identifier
///
/// Tier: T2-P (cross-domain primitive — wraps String)
/// Grounds to T1 State (ς): identity token
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CaseId(pub String);

impl CaseId {
    /// Create a new case ID
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the inner string
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for CaseId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Signal identifier
///
/// Tier: T2-P (cross-domain primitive — wraps String)
/// Grounds to T1 State (ς): identity token
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SignalId(pub String);

impl SignalId {
    /// Create a new signal ID
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Create from drug-event pair
    #[must_use]
    pub fn from_pair(drug: &str, event: &str) -> Self {
        Self(format!("{drug}::{event}"))
    }

    /// Get the inner string
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SignalId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Report from a decay operation
///
/// Tier: T2-C (cross-domain composite — aggregates decay metrics)
/// Grounds to T1 Sequence (σ): decay summary order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecayReport {
    /// Name of the function that ran decay
    pub function_name: String,
    /// Number of items that had decay applied
    pub items_decayed: usize,
    /// Number of items below the relevance cutoff after decay
    pub items_below_threshold: usize,
    /// Minimum confidence after decay (None if no items)
    pub min_confidence: Option<f64>,
    /// Maximum confidence after decay (None if no items)
    pub max_confidence: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn similarity_clamps() {
        assert!((Similarity::new(1.5).value() - 1.0).abs() < f64::EPSILON);
        assert!((Similarity::new(-0.5).value() - 0.0).abs() < f64::EPSILON);
        assert!((Similarity::new(0.7).value() - 0.7).abs() < f64::EPSILON);
    }

    #[test]
    fn relevance_clamps() {
        assert!((Relevance::new(2.0).value() - 1.0).abs() < f64::EPSILON);
        assert!((Relevance::new(-1.0).value() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn half_life_positive() {
        assert!((HalfLife::new(-5.0).days() - 30.0).abs() < f64::EPSILON);
        assert!((HalfLife::new(0.0).days() - 30.0).abs() < f64::EPSILON);
        assert!((HalfLife::new(7.0).days() - 7.0).abs() < f64::EPSILON);
    }

    #[test]
    fn signal_id_from_pair() {
        let id = SignalId::from_pair("aspirin", "headache");
        assert_eq!(id.as_str(), "aspirin::headache");
    }
}
