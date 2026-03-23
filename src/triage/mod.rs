//! Signal Triage function
//!
//! Exponential decay priority queue with reinforcement learning.
//! Signals decay over time; new evidence reinforces them.
//!
//! Tier: T3 (domain-specific PV function)
//! Grounds to: T1 Sequence (σ) and Persistence (π) (time-ordered decay)

pub mod classifier;
pub mod decay;
pub mod queue;
pub mod types;

use nexcore_lex_primitiva::LexPrimitiva;

use crate::error::Result;
use crate::traits::AlgovigilanceFunction;
use crate::types::DecayReport;

use self::queue::SignalQueue;
use self::types::{ReinforcementEvent, SignalInput, TriageConfig, TriageResult};

/// Signal Triage function
///
/// Manages a priority queue of signals with decay and reinforcement.
pub struct TriageFunction {
    /// The signal queue
    queue: SignalQueue,
    /// Configuration
    config: TriageConfig,
}

impl TriageFunction {
    /// Create with default config
    #[must_use]
    pub fn new() -> Self {
        let config = TriageConfig::default();
        Self {
            queue: SignalQueue::new(&config),
            config,
        }
    }

    /// Create with custom config
    #[must_use]
    pub fn with_config(config: TriageConfig) -> Self {
        Self {
            queue: SignalQueue::new(&config),
            config,
        }
    }

    /// Get a reference to the internal queue
    #[must_use]
    pub fn queue(&self) -> &SignalQueue {
        &self.queue
    }

    /// Get a mutable reference to the internal queue
    pub fn queue_mut(&mut self) -> &mut SignalQueue {
        &mut self.queue
    }
}

impl Default for TriageFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl AlgovigilanceFunction for TriageFunction {
    type Input = Vec<SignalInput>;
    type Output = TriageResult;
    type Feedback = ReinforcementEvent;

    fn process(&self, input: &Self::Input) -> Result<Self::Output> {
        // Create a temporary queue to process input without mutating self
        let mut temp_queue = self.queue.clone();

        for signal_input in input {
            temp_queue.insert(signal_input);
        }

        let cutoff_val = self.config.cutoff_relevance;
        let all_signals = temp_queue.signals().to_vec();
        let total = all_signals.len();

        let (active, decayed): (Vec<_>, Vec<_>) = all_signals
            .into_iter()
            .partition(|s| s.current_relevance.value() >= cutoff_val);

        Ok(TriageResult {
            active_signals: active,
            decayed_signals: decayed,
            total,
        })
    }

    fn learn(&mut self, feedback: &Self::Feedback) -> Result<()> {
        self.queue
            .reinforce(&feedback.signal_id, feedback.new_case_count);
        Ok(())
    }

    fn decay(&mut self, elapsed_days: f64) -> Result<DecayReport> {
        let before_count = self.queue.len();
        let removed = self.queue.decay_all(elapsed_days);

        let signals = self.queue.signals();
        let (min_conf, max_conf) = if signals.is_empty() {
            (None, None)
        } else {
            let min = signals
                .iter()
                .map(|s| s.current_relevance.value())
                .fold(f64::INFINITY, f64::min);
            let max = signals
                .iter()
                .map(|s| s.current_relevance.value())
                .fold(f64::NEG_INFINITY, f64::max);
            (Some(min), Some(max))
        };

        Ok(DecayReport {
            function_name: "triage".to_string(),
            items_decayed: before_count,
            items_below_threshold: removed,
            min_confidence: min_conf,
            max_confidence: max_conf,
        })
    }

    fn name(&self) -> &'static str {
        "signal_triage"
    }

    fn t1_grounding(&self) -> LexPrimitiva {
        LexPrimitiva::Sequence
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SignalId;

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
    fn process_creates_signals() {
        let func = TriageFunction::new();
        let inputs = vec![
            make_input("aspirin", "bleeding", 0.8),
            make_input("ibuprofen", "nausea", 0.6),
        ];
        let result = func.process(&inputs).expect("process");
        assert_eq!(result.total, 2);
        assert_eq!(result.active_signals.len(), 2);
    }

    #[test]
    fn learn_reinforces() {
        let mut func = TriageFunction::new();
        let inputs = vec![make_input("aspirin", "bleeding", 0.8)];

        // Insert into actual queue
        for input in &inputs {
            func.queue_mut().insert(input);
        }

        // Decay
        func.decay(30.0).expect("decay");
        let before = func.queue().signals()[0].current_relevance.value();

        // Reinforce
        func.learn(
            &(ReinforcementEvent {
                signal_id: SignalId::from_pair("aspirin", "bleeding"),
                new_case_count: 5,
            }),
        )
        .expect("learn");
        let after = func.queue().signals()[0].current_relevance.value();
        assert!(after > before);
    }

    #[test]
    fn decay_report() {
        let mut func = TriageFunction::new();
        func.queue_mut()
            .insert(&make_input("aspirin", "bleeding", 0.8));
        func.queue_mut()
            .insert(&make_input("ibuprofen", "nausea", 0.6));

        let report = func.decay(15.0).expect("decay");
        assert_eq!(report.items_decayed, 2);
        assert!(report.min_confidence.is_some());
        assert!(report.max_confidence.is_some());
    }

    #[test]
    fn t1_grounding_is_sequence() {
        let func = TriageFunction::new();
        assert_eq!(func.t1_grounding(), LexPrimitiva::Sequence);
    }
}
