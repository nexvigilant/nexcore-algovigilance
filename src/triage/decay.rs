//! Exponential decay engine
//!
//! Implements `confidence * 0.5^(elapsed_days / half_life)` for time-based
//! relevance decay. Same formula as `nexcore-brain::implicit::Pattern`.
//!
//! Tier: T2-P (Cross-domain atomic transformation)
//! Grounds to: T1 Persistence (π), Mapping (μ), and Quantity (N) (time-ordered transformation)

use nexcore_chrono::DateTime;

use super::types::TriagedSignal;
use crate::types::Relevance;

/// Apply exponential decay to a confidence value
///
/// Formula: `confidence * 0.5^(elapsed_days / half_life)`
///
/// CONFESSION: Commandment 4 — WRAP
/// Returns naked f64: pure math primitive. Callers wrap into `Relevance`.
/// I have confessed. The record stands.
#[must_use]
pub fn apply_decay(confidence: f64, elapsed_days: f64, half_life: f64) -> f64 {
    if half_life <= 0.0 || elapsed_days <= 0.0 {
        return confidence;
    }
    confidence * 0.5_f64.powf(elapsed_days / half_life)
}

/// Reinforce a signal with new evidence
///
/// Restores confidence toward original and resets last_reinforced timestamp.
pub fn reinforce(signal: &mut TriagedSignal, new_cases: u32) {
    // Boost: restore toward original_confidence proportional to new evidence
    let boost = (signal.original_confidence - signal.current_relevance.value())
        * (1.0 - 0.5_f64.powf(f64::from(new_cases)));
    let new_relevance = (signal.current_relevance.value() + boost).min(1.0);
    signal.current_relevance = Relevance::new(new_relevance);
    signal.last_reinforced = DateTime::now();
    signal.reinforcement_count += new_cases;
}

/// Compute elapsed days between two timestamps
#[must_use]
pub fn elapsed_days(from: DateTime, to: DateTime) -> f64 {
    let duration = to.signed_duration_since(from);
    duration.num_seconds() as f64 / 86_400.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decay_at_half_life() {
        let result = apply_decay(1.0, 30.0, 30.0);
        assert!((result - 0.5).abs() < 0.001);
    }

    #[test]
    fn decay_at_two_half_lives() {
        let result = apply_decay(1.0, 60.0, 30.0);
        assert!((result - 0.25).abs() < 0.001);
    }

    #[test]
    fn decay_at_three_half_lives() {
        let result = apply_decay(1.0, 90.0, 30.0);
        assert!((result - 0.125).abs() < 0.001);
    }

    #[test]
    fn no_decay_at_zero_days() {
        let result = apply_decay(0.8, 0.0, 30.0);
        assert!((result - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn no_decay_negative_days() {
        let result = apply_decay(0.8, -5.0, 30.0);
        assert!((result - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn zero_half_life_no_decay() {
        let result = apply_decay(0.8, 10.0, 0.0);
        assert!((result - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn reinforce_restores_confidence() {
        use crate::types::SignalId;

        let mut signal = TriagedSignal {
            signal_id: SignalId::from_pair("aspirin", "bleeding"),
            drug: "aspirin".to_string(),
            event: "bleeding".to_string(),
            prr: 3.0,
            ror: 2.5,
            original_confidence: 0.9,
            current_relevance: Relevance::new(0.3),
            last_reinforced: DateTime::now(),
            first_detected: DateTime::now(),
            reinforcement_count: 0,
        };

        reinforce(&mut signal, 5);
        assert!(signal.current_relevance.value() > 0.3);
        assert!(signal.current_relevance.value() <= 0.9);
        assert_eq!(signal.reinforcement_count, 5);
    }
}
