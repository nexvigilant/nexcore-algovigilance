//! Priority queue with relevance sorting
//!
//! Maintains signals sorted by current_relevance (descending).
//! Supports insert, decay_all, top_n, and reinforce operations.
//!
//! Tier: T2-C (cross-domain composite — sorted collection + decay)
//! Grounds to: T1::Sequence (ordered by relevance)

use nexcore_chrono::DateTime;

use super::decay;
use super::types::{SignalInput, TriageConfig, TriagedSignal};
use crate::types::{HalfLife, Relevance, SignalId};

/// Signal priority queue ordered by relevance
#[derive(Debug, Clone)]
pub struct SignalQueue {
    /// Signals in the queue (sorted by relevance descending)
    signals: Vec<TriagedSignal>,
    /// Half-life for decay
    half_life: HalfLife,
    /// Minimum relevance cutoff
    cutoff: Relevance,
    /// Maximum queue size
    max_size: usize,
}

impl SignalQueue {
    /// Create a new empty queue with config
    #[must_use]
    pub fn new(config: &TriageConfig) -> Self {
        Self {
            signals: Vec::new(),
            half_life: HalfLife::new(config.half_life_days),
            cutoff: Relevance::new(config.cutoff_relevance),
            max_size: config.max_queue_size,
        }
    }

    /// Insert a new signal from raw input
    pub fn insert(&mut self, input: &SignalInput) {
        let now = DateTime::now();
        let signal = TriagedSignal {
            signal_id: SignalId::from_pair(&input.drug, &input.event),
            drug: input.drug.clone(),
            event: input.event.clone(),
            prr: input.prr,
            ror: input.ror,
            original_confidence: input.confidence,
            current_relevance: Relevance::new(input.confidence),
            last_reinforced: now,
            first_detected: now,
            reinforcement_count: 0,
        };

        // Check if signal already exists — reinforce instead
        for existing in &mut self.signals {
            if existing.signal_id == signal.signal_id {
                decay::reinforce(existing, 1);
                self.sort();
                return;
            }
        }

        self.signals.push(signal);
        self.sort();

        // Trim to max size
        if self.signals.len() > self.max_size {
            self.signals.truncate(self.max_size);
        }
    }

    /// Insert a pre-built TriagedSignal directly
    pub fn insert_signal(&mut self, signal: TriagedSignal) {
        for existing in &mut self.signals {
            if existing.signal_id == signal.signal_id {
                decay::reinforce(existing, 1);
                self.sort();
                return;
            }
        }

        self.signals.push(signal);
        self.sort();

        if self.signals.len() > self.max_size {
            self.signals.truncate(self.max_size);
        }
    }

    /// Apply decay to all signals, remove those below cutoff
    ///
    /// Returns count of signals removed.
    pub fn decay_all(&mut self, elapsed_days: f64) -> usize {
        let hl = self.half_life.days();
        let cutoff_val = self.cutoff.value();

        for signal in &mut self.signals {
            let decayed = decay::apply_decay(signal.current_relevance.value(), elapsed_days, hl);
            signal.current_relevance = Relevance::new(decayed);
        }

        let before = self.signals.len();
        self.signals
            .retain(|s| s.current_relevance.value() >= cutoff_val);
        let removed = before - self.signals.len();

        self.sort();
        removed
    }

    /// Get top N signals by relevance
    #[must_use]
    pub fn top_n(&self, n: usize) -> &[TriagedSignal] {
        let end = n.min(self.signals.len());
        &self.signals[..end]
    }

    /// Reinforce a signal by ID
    ///
    /// Returns true if signal was found and reinforced.
    pub fn reinforce(&mut self, signal_id: &SignalId, new_cases: u32) -> bool {
        for signal in &mut self.signals {
            if &signal.signal_id == signal_id {
                decay::reinforce(signal, new_cases);
                self.sort();
                return true;
            }
        }
        false
    }

    /// Get all signals (sorted by relevance descending)
    #[must_use]
    pub fn signals(&self) -> &[TriagedSignal] {
        &self.signals
    }

    /// Get queue length
    #[must_use]
    pub fn len(&self) -> usize {
        self.signals.len()
    }

    /// Check if queue is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.signals.is_empty()
    }

    /// Find a signal by ID
    #[must_use]
    pub fn find(&self, signal_id: &SignalId) -> Option<&TriagedSignal> {
        self.signals.iter().find(|s| &s.signal_id == signal_id)
    }

    /// Sort by relevance descending
    fn sort(&mut self) {
        self.signals.sort_by(|a, b| {
            b.current_relevance
                .value()
                .partial_cmp(&a.current_relevance.value())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> TriageConfig {
        TriageConfig::default()
    }

    fn make_input(drug: &str, event: &str, confidence: f64) -> SignalInput {
        SignalInput {
            drug: drug.to_string(),
            event: event.to_string(),
            prr: 3.0,
            ror: 2.5,
            confidence,
        }
    }

    #[test]
    fn empty_queue() {
        let queue = SignalQueue::new(&default_config());
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn insert_and_retrieve() {
        let mut queue = SignalQueue::new(&default_config());
        queue.insert(&make_input("aspirin", "bleeding", 0.8));
        assert_eq!(queue.len(), 1);
        assert_eq!(queue.top_n(1)[0].drug, "aspirin");
    }

    #[test]
    fn ordering_by_relevance() {
        let mut queue = SignalQueue::new(&default_config());
        queue.insert(&make_input("aspirin", "bleeding", 0.5));
        queue.insert(&make_input("ibuprofen", "nausea", 0.9));
        queue.insert(&make_input("warfarin", "rash", 0.7));

        let top = queue.top_n(3);
        assert!(top[0].current_relevance.value() >= top[1].current_relevance.value());
        assert!(top[1].current_relevance.value() >= top[2].current_relevance.value());
    }

    #[test]
    fn decay_removes_below_cutoff() {
        let config = TriageConfig {
            half_life_days: 30.0,
            cutoff_relevance: 0.3,
            ..default_config()
        };
        let mut queue = SignalQueue::new(&config);
        queue.insert(&make_input("aspirin", "bleeding", 0.5));

        // After 60 days: 0.5 * 0.25 = 0.125 < 0.3 cutoff
        let removed = queue.decay_all(60.0);
        assert_eq!(removed, 1);
        assert!(queue.is_empty());
    }

    #[test]
    fn reinforce_boosts_relevance() {
        let mut queue = SignalQueue::new(&default_config());
        queue.insert(&make_input("aspirin", "bleeding", 0.8));

        // Decay first
        queue.decay_all(30.0);
        let before = queue.signals()[0].current_relevance.value();

        // Reinforce
        let id = SignalId::from_pair("aspirin", "bleeding");
        assert!(queue.reinforce(&id, 3));
        let after = queue.signals()[0].current_relevance.value();
        assert!(after > before);
    }

    #[test]
    fn duplicate_insert_reinforces() {
        let mut queue = SignalQueue::new(&default_config());
        queue.insert(&make_input("aspirin", "bleeding", 0.8));
        queue.insert(&make_input("aspirin", "bleeding", 0.8));
        // Should still be 1 signal, but reinforced
        assert_eq!(queue.len(), 1);
        assert!(queue.signals()[0].reinforcement_count > 0);
    }

    #[test]
    fn find_by_id() {
        let mut queue = SignalQueue::new(&default_config());
        queue.insert(&make_input("aspirin", "bleeding", 0.8));
        let id = SignalId::from_pair("aspirin", "bleeding");
        assert!(queue.find(&id).is_some());
        let missing = SignalId::from_pair("unknown", "thing");
        assert!(queue.find(&missing).is_none());
    }
}
