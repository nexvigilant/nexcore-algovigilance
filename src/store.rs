//! Federated persistence store
//!
//! Two backends:
//! 1. Brain implicit (`~/.claude/implicit/`) — cross-session patterns via `ImplicitKnowledge`
//! 2. Dedicated (`~/nexcore/algovigilance/`) — domain-specific state
//!
//! Tier: T2-C (cross-domain composite — aggregates two persistence backends)
//! Grounds to: T1::State (encapsulated persistence context)

use nexcore_fs::dirs;
use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::Result;

/// Synonym pair learned from dedup decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynonymEntry {
    /// First term
    pub term_a: String,
    /// Second term (equivalent)
    pub term_b: String,
    /// Confidence in synonym relationship [0.0, 1.0]
    pub confidence: f64,
    /// Number of times reinforced
    pub reinforcement_count: u32,
}

/// Persisted state for the algovigilance store
#[derive(Debug, Default, Serialize, Deserialize)]
struct DedicatedState {
    /// Learned synonym pairs from dedup
    dedup_synonyms: Vec<SynonymEntry>,
    /// Signal queue state (JSON blob per drug)
    signal_queues: HashMap<String, serde_json::Value>,
}

/// Federated persistence store
///
/// Manages Brain implicit (cross-session) and dedicated (domain-specific) backends.
pub struct AlgovigilanceStore {
    /// Path to dedicated store directory
    dedicated_path: PathBuf,
    /// In-memory dedicated state
    state: DedicatedState,
}

impl AlgovigilanceStore {
    /// Initialize the store, creating directories and loading state
    pub fn init() -> Result<Self> {
        let dedicated_path = Self::dedicated_dir();
        std::fs::create_dir_all(&dedicated_path)?;

        let state = Self::load_dedicated_state(&dedicated_path)?;

        Ok(Self {
            dedicated_path,
            state,
        })
    }

    /// Get the dedicated store directory
    fn dedicated_dir() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("nexcore")
            .join("algovigilance")
    }

    /// Load dedicated state from disk
    fn load_dedicated_state(path: &std::path::Path) -> Result<DedicatedState> {
        let state_file = path.join("state.json");
        if state_file.exists() {
            let content = std::fs::read_to_string(&state_file)?;
            serde_json::from_str(&content).map_err(Into::into)
        } else {
            Ok(DedicatedState::default())
        }
    }

    /// Save dedicated state to disk
    pub fn save(&self) -> Result<()> {
        let state_file = self.dedicated_path.join("state.json");
        let content = serde_json::to_string_pretty(&self.state)?;
        std::fs::write(&state_file, content)?;
        Ok(())
    }

    /// Get the brain implicit directory
    fn implicit_dir() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".claude")
            .join("implicit")
    }

    /// Save a pattern to Brain implicit store (cross-session learning)
    pub fn save_to_brain(&self, key: &str, value: &serde_json::Value) -> Result<()> {
        let implicit_dir = Self::implicit_dir();
        std::fs::create_dir_all(&implicit_dir)?;

        // Load existing preferences
        let prefs_file = implicit_dir.join("preferences.json");
        let mut prefs: HashMap<String, serde_json::Value> = if prefs_file.exists() {
            let content = std::fs::read_to_string(&prefs_file)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            HashMap::new()
        };

        prefs.insert(format!("algovigil.{key}"), value.clone());
        let content = serde_json::to_string_pretty(&prefs)?;
        std::fs::write(&prefs_file, content)?;
        Ok(())
    }

    // ── Synonym operations ──────────────────────────────────────────

    /// Add or reinforce a synonym pair
    pub fn add_synonym(&mut self, term_a: &str, term_b: &str, confidence: f64) {
        // Check for existing pair (order-independent)
        for entry in &mut self.state.dedup_synonyms {
            if (entry.term_a == term_a && entry.term_b == term_b)
                || (entry.term_a == term_b && entry.term_b == term_a)
            {
                entry.reinforcement_count += 1;
                // Asymptotic growth: c + (1-c) * 0.1
                entry.confidence += (1.0 - entry.confidence) * 0.1;
                entry.confidence = entry.confidence.min(1.0);
                return;
            }
        }

        self.state.dedup_synonyms.push(SynonymEntry {
            term_a: term_a.to_string(),
            term_b: term_b.to_string(),
            confidence: confidence.clamp(0.0, 1.0),
            reinforcement_count: 1,
        });
    }

    /// Get all learned synonyms
    #[must_use]
    pub fn synonyms(&self) -> &[SynonymEntry] {
        &self.state.dedup_synonyms
    }

    /// Get synonym count
    #[must_use]
    pub fn synonym_count(&self) -> usize {
        self.state.dedup_synonyms.len()
    }

    /// Decay synonym confidence and prune below threshold
    pub fn decay_synonyms(&mut self, elapsed_days: f64, half_life: f64, cutoff: f64) -> usize {
        let factor = (0.5_f64).powf(elapsed_days / half_life);
        for entry in &mut self.state.dedup_synonyms {
            entry.confidence *= factor;
        }
        let before = self.state.dedup_synonyms.len();
        self.state.dedup_synonyms.retain(|e| e.confidence >= cutoff);
        before - self.state.dedup_synonyms.len()
    }

    // ── Signal queue persistence ────────────────────────────────────

    /// Save signal queue state for a drug
    pub fn save_signal_queue(&mut self, drug: &str, queue_json: serde_json::Value) -> Result<()> {
        self.state
            .signal_queues
            .insert(drug.to_lowercase(), queue_json);
        Ok(())
    }

    /// Load signal queue state for a drug
    #[must_use]
    pub fn load_signal_queue(&self, drug: &str) -> Option<&serde_json::Value> {
        self.state.signal_queues.get(&drug.to_lowercase())
    }

    /// Get total queue count across all drugs
    #[must_use]
    pub fn queue_count(&self) -> usize {
        self.state.signal_queues.len()
    }

    /// Health check: returns (synonym_count, queue_count, store_path_exists)
    #[must_use]
    pub fn health(&self) -> (usize, usize, bool) {
        (
            self.synonym_count(),
            self.queue_count(),
            self.dedicated_path.exists(),
        )
    }
}

impl AlgovigilanceStore {
    /// Create with a custom path (for testing)
    #[cfg(test)]
    pub fn with_path(path: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&path)?;
        Ok(Self {
            dedicated_path: path,
            state: DedicatedState::default(),
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn test_store(dir: &std::path::Path) -> AlgovigilanceStore {
        AlgovigilanceStore::with_path(dir.to_path_buf()).expect("store init")
    }

    #[test]
    fn test_store_init_and_save() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut store = test_store(dir.path());

        store.add_synonym("headache", "cephalgia", 0.9);
        assert_eq!(store.synonym_count(), 1);

        store.save().expect("save");

        let loaded =
            AlgovigilanceStore::load_dedicated_state(&dir.path().to_path_buf()).expect("load");
        assert_eq!(loaded.dedup_synonyms.len(), 1);
    }

    #[test]
    fn test_synonym_reinforcement() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut store = test_store(dir.path());

        store.add_synonym("nausea", "emesis", 0.7);
        store.add_synonym("emesis", "nausea", 0.7); // reversed order — should reinforce
        assert_eq!(store.synonym_count(), 1);
        assert!(store.synonyms()[0].confidence > 0.7);
        assert_eq!(store.synonyms()[0].reinforcement_count, 2);
    }

    #[test]
    fn test_synonym_decay() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut store = test_store(dir.path());

        store.add_synonym("rash", "dermatitis", 0.5);
        let pruned = store.decay_synonyms(60.0, 30.0, 0.2);
        // After 60 days with 30-day half-life: 0.5 * 0.25 = 0.125 < 0.2 cutoff
        assert_eq!(pruned, 1);
        assert_eq!(store.synonym_count(), 0);
    }

    #[test]
    fn test_signal_queue_persistence() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut store = test_store(dir.path());

        let queue_data = serde_json::json!({"signals": [{"drug": "aspirin", "event": "bleeding"}]});
        store
            .save_signal_queue("aspirin", queue_data.clone())
            .expect("save queue");

        assert_eq!(store.load_signal_queue("aspirin"), Some(&queue_data));
        assert!(store.load_signal_queue("unknown").is_none());
    }

    #[test]
    fn test_health() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = test_store(dir.path());
        let (syns, queues, exists) = store.health();
        assert_eq!(syns, 0);
        assert_eq!(queues, 0);
        assert!(exists);
    }
}
