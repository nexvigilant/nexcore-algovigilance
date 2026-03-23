//! # Decision Tree Signal Classifier
//!
//! Classifies triaged signals into urgency levels using a trained CART tree.
//! Feature vector: [prr, ror, confidence, reinforcement_count, relevance]
//!
//! Tier: T2-C (composes T1 mapping with T3 triage types)
//! Grounds to: T1::Mapping (μ) — signal features → urgency

use nexcore_dtree::prelude::*;

use super::types::TriagedSignal;

// ============================================================================
// Urgency levels
// ============================================================================

/// Signal urgency classification labels
pub mod urgency {
    /// Immediate regulatory action needed
    pub const CRITICAL: &str = "Critical";
    /// Requires expedited review
    pub const HIGH: &str = "High";
    /// Standard review queue
    pub const MEDIUM: &str = "Medium";
    /// Monitor only
    pub const LOW: &str = "Low";
    /// Below noise threshold — candidate for removal
    pub const NOISE: &str = "Noise";
}

/// Feature indices for triage classification
pub mod feature_index {
    /// PRR from signal detection
    pub const PRR: usize = 0;
    /// ROR from signal detection
    pub const ROR: usize = 1;
    /// Original confidence at detection time
    pub const CONFIDENCE: usize = 2;
    /// Number of reinforcement events (evidence accumulation)
    pub const REINFORCEMENTS: usize = 3;
    /// Current decay-adjusted relevance
    pub const RELEVANCE: usize = 4;
    /// Total feature count
    pub const COUNT: usize = 5;
}

/// Feature names for explainability
const FEATURE_NAMES: [&str; feature_index::COUNT] =
    ["prr", "ror", "confidence", "reinforcements", "relevance"];

// ============================================================================
// Feature extraction
// ============================================================================

/// Extract feature vector from a `TriagedSignal`.
///
/// Tier: T1 Mapping (μ) — domain → numeric
#[must_use]
pub fn extract_features(signal: &TriagedSignal) -> Vec<Feature> {
    vec![
        Feature::Continuous(signal.prr),
        Feature::Continuous(signal.ror),
        Feature::Continuous(signal.original_confidence),
        Feature::Continuous(f64::from(signal.reinforcement_count)),
        Feature::Continuous(signal.current_relevance.value()),
    ]
}

/// Extract raw f64 values (for batch training)
#[must_use]
pub fn extract_raw(signal: &TriagedSignal) -> Vec<f64> {
    vec![
        signal.prr,
        signal.ror,
        signal.original_confidence,
        f64::from(signal.reinforcement_count),
        signal.current_relevance.value(),
    ]
}

// ============================================================================
// Urgency Classifier
// ============================================================================

/// Urgency classification result
#[derive(Debug, Clone)]
pub struct UrgencyClassification {
    /// Predicted urgency label
    pub urgency: String,
    /// Tree confidence (0.0-1.0)
    pub confidence: f64,
    /// Decision path for explainability
    pub path: Vec<String>,
}

/// Decision tree-backed signal urgency classifier.
///
/// Tier: T2-C (composed mapping + state)
pub struct UrgencyClassifier {
    /// The trained decision tree
    tree: DecisionTree,
    /// Minimum confidence to return a classification
    min_confidence: f64,
}

impl UrgencyClassifier {
    /// Train from historical signal/urgency pairs.
    ///
    /// # Errors
    /// Returns `Err` if training data is empty or training fails.
    pub fn train(
        signals: &[TriagedSignal],
        labels: &[&str],
        config: TreeConfig,
    ) -> Result<Self, nexcore_dtree::train::TrainError> {
        if signals.is_empty() || signals.len() != labels.len() {
            return Err(nexcore_dtree::train::TrainError::EmptyData);
        }

        let features: Vec<Vec<Feature>> = signals.iter().map(|s| extract_features(s)).collect();

        let string_labels: Vec<String> = labels.iter().map(|l| (*l).to_string()).collect();

        let mut tree = fit(&features, &string_labels, config)?;
        tree.set_feature_names(FEATURE_NAMES.iter().map(|s| (*s).to_string()).collect());

        Ok(Self {
            tree,
            min_confidence: 0.5,
        })
    }

    /// Set minimum confidence threshold.
    #[must_use]
    pub fn with_min_confidence(mut self, threshold: f64) -> Self {
        self.min_confidence = threshold.clamp(0.0, 1.0);
        self
    }

    /// Classify a signal's urgency.
    ///
    /// Returns `None` if confidence is below threshold.
    #[must_use]
    pub fn classify(&self, signal: &TriagedSignal) -> Option<UrgencyClassification> {
        let features = extract_features(signal);
        let result = predict(&self.tree, &features).ok()?;

        if result.confidence.value() < self.min_confidence {
            return None;
        }

        let path: Vec<String> = result.path.iter().map(|step| format!("{step}")).collect();

        Some(UrgencyClassification {
            urgency: result.prediction,
            confidence: result.confidence.value(),
            path,
        })
    }

    /// Classify a batch of signals.
    #[must_use]
    pub fn classify_batch(&self, signals: &[TriagedSignal]) -> Vec<Option<UrgencyClassification>> {
        signals.iter().map(|s| self.classify(s)).collect()
    }

    /// Get feature importance scores.
    #[must_use]
    pub fn importance(&self) -> Vec<FeatureImportance> {
        feature_importance(&self.tree)
    }

    /// Get the underlying tree reference.
    #[must_use]
    pub fn tree(&self) -> &DecisionTree {
        &self.tree
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Relevance, SignalId};
    use nexcore_chrono::DateTime;

    fn make_signal(
        prr: f64,
        ror: f64,
        conf: f64,
        reinforcements: u32,
        relevance: f64,
    ) -> TriagedSignal {
        TriagedSignal {
            signal_id: SignalId::from_pair("drug", "event"),
            drug: "drug".to_string(),
            event: "event".to_string(),
            prr,
            ror,
            original_confidence: conf,
            current_relevance: Relevance::new(relevance),
            last_reinforced: DateTime::now(),
            first_detected: DateTime::now(),
            reinforcement_count: reinforcements,
        }
    }

    fn training_set() -> (Vec<TriagedSignal>, Vec<&'static str>) {
        let signals = vec![
            make_signal(5.0, 4.0, 0.9, 10, 0.95),
            make_signal(4.5, 3.5, 0.85, 8, 0.90),
            make_signal(2.0, 1.5, 0.6, 3, 0.60),
            make_signal(1.8, 1.2, 0.5, 2, 0.50),
            make_signal(0.5, 0.4, 0.2, 0, 0.15),
            make_signal(0.3, 0.2, 0.1, 0, 0.08),
        ];
        let labels = vec![
            urgency::CRITICAL,
            urgency::CRITICAL,
            urgency::MEDIUM,
            urgency::MEDIUM,
            urgency::NOISE,
            urgency::NOISE,
        ];
        (signals, labels)
    }

    #[test]
    fn extract_features_count() {
        let signal = make_signal(3.0, 2.0, 0.7, 5, 0.8);
        let features = extract_features(&signal);
        assert_eq!(features.len(), feature_index::COUNT);
    }

    #[test]
    fn train_and_classify_critical() {
        let (signals, labels) = training_set();
        let classifier = UrgencyClassifier::train(&signals, &labels, TreeConfig::default())
            .ok()
            .expect("train ok");

        let strong = make_signal(5.5, 4.5, 0.95, 12, 0.98);
        let result = classifier.classify(&strong);
        assert!(result.is_some());
        assert_eq!(result.expect("classified").urgency, urgency::CRITICAL);
    }

    #[test]
    fn train_and_classify_noise() {
        let (signals, labels) = training_set();
        let classifier = UrgencyClassifier::train(&signals, &labels, TreeConfig::default())
            .ok()
            .expect("train ok");

        let weak = make_signal(0.2, 0.1, 0.05, 0, 0.05);
        let result = classifier.classify(&weak);
        assert!(result.is_some());
        assert_eq!(result.expect("classified").urgency, urgency::NOISE);
    }

    #[test]
    fn classify_batch_matches_individual() {
        let (signals, labels) = training_set();
        let classifier = UrgencyClassifier::train(&signals, &labels, TreeConfig::default())
            .ok()
            .expect("train ok");

        let test = vec![
            make_signal(5.0, 4.0, 0.9, 10, 0.95),
            make_signal(0.3, 0.2, 0.1, 0, 0.08),
        ];

        let batch = classifier.classify_batch(&test);
        assert_eq!(batch.len(), 2);
        assert!(batch[0].is_some());
        assert!(batch[1].is_some());
    }

    #[test]
    fn importance_is_populated() {
        let (signals, labels) = training_set();
        let classifier = UrgencyClassifier::train(&signals, &labels, TreeConfig::default())
            .ok()
            .expect("train ok");

        let imp = classifier.importance();
        assert!(!imp.is_empty());
    }
}
