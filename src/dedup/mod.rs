//! ICSR Deduplication function
//!
//! Compares ICSR narratives via Jaccard tokenization, detects duplicates,
//! learns synonym pairs, and integrates with FAERS for batch dedup.
//!
//! Tier: T3 (domain-specific PV function)
//! Grounds to: T1::Mapping (narrative_set → deduplicated_set)

pub mod tokenizer;
pub mod types;

use std::collections::HashSet;

use rayon::prelude::*;

use crate::error::Result;
use crate::store::AlgovigilanceStore;
use crate::traits::AlgovigilanceFunction;
use crate::types::{CaseId, DecayReport, Similarity};

use nexcore_proof_of_meaning::synonymy::SynonymRegistry;

use self::tokenizer::narrative_similarity_with_registry;
use self::types::{CasePair, DedupConfig, DeduplicationResult, IcsrNarrative, SynonymPair};

/// ICSR Deduplication function
///
/// Pairwise narrative comparison with learned synonym boosting.
pub struct DedupFunction {
    /// Configuration
    config: DedupConfig,
    /// Curated POM synonym registry (always active)
    registry: SynonymRegistry,
    /// Learned synonym pairs (canonical, variant) — gated by use_learned_synonyms
    synonyms: Vec<(String, String)>,
}

impl DedupFunction {
    /// Create with default config
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: DedupConfig::default(),
            registry: SynonymRegistry::pv_standard(),
            synonyms: Vec::new(),
        }
    }

    /// Create with custom config
    #[must_use]
    pub fn with_config(config: DedupConfig) -> Self {
        Self {
            config,
            registry: SynonymRegistry::pv_standard(),
            synonyms: Vec::new(),
        }
    }

    /// Load synonyms from store
    pub fn load_synonyms(&mut self, store: &AlgovigilanceStore) {
        self.synonyms = store
            .synonyms()
            .iter()
            .map(|s| (s.term_a.clone(), s.term_b.clone()))
            .collect();
    }

    /// Compare two narratives directly
    ///
    /// Always uses POM `SynonymRegistry` for curated baseline resolution.
    /// Learned synonyms layer on top when `use_learned_synonyms` is enabled.
    #[must_use]
    pub fn compare_pair(&self, a: &str, b: &str) -> Similarity {
        let learned = if self.config.use_learned_synonyms {
            &self.synonyms[..]
        } else {
            &[]
        };
        narrative_similarity_with_registry(a, b, &self.registry, learned)
    }

    /// Deduplicate a batch of ICSR narratives
    fn deduplicate_batch(&self, narratives: &[IcsrNarrative]) -> DeduplicationResult {
        let n = narratives.len();
        if n <= 1 {
            return DeduplicationResult {
                unique_cases: narratives.iter().map(|n| n.case_id.clone()).collect(),
                duplicate_pairs: Vec::new(),
                total_input: n,
                total_unique: n,
                synonym_pairs_learned: 0,
            };
        }

        // Generate all pairs for comparison
        let pairs: Vec<(usize, usize)> = (0..n)
            .flat_map(|i| (i + 1..n).map(move |j| (i, j)))
            .collect();

        // Always use POM registry; learned synonyms gated by config flag
        let threshold = self.config.similarity_threshold;
        let learned: &[(String, String)] = if self.config.use_learned_synonyms {
            &self.synonyms
        } else {
            &[]
        };
        let registry = &self.registry;

        let case_pairs: Vec<CasePair> = if n > 100 {
            pairs
                .par_iter()
                .filter_map(|&(i, j)| {
                    let sim = narrative_similarity_with_registry(
                        &narratives[i].narrative_text,
                        &narratives[j].narrative_text,
                        registry,
                        learned,
                    );

                    if sim.value() >= threshold {
                        Some(CasePair {
                            case_a: narratives[i].case_id.clone(),
                            case_b: narratives[j].case_id.clone(),
                            similarity: sim,
                            is_duplicate: true,
                        })
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            pairs
                .iter()
                .filter_map(|&(i, j)| {
                    let sim = narrative_similarity_with_registry(
                        &narratives[i].narrative_text,
                        &narratives[j].narrative_text,
                        registry,
                        learned,
                    );

                    if sim.value() >= threshold {
                        Some(CasePair {
                            case_a: narratives[i].case_id.clone(),
                            case_b: narratives[j].case_id.clone(),
                            similarity: sim,
                            is_duplicate: true,
                        })
                    } else {
                        None
                    }
                })
                .collect()
        };

        // Determine unique cases (not the "b" side of any duplicate pair)
        let duplicate_b_ids: HashSet<&str> = case_pairs.iter().map(|p| p.case_b.as_str()).collect();

        let unique_cases: Vec<CaseId> = narratives
            .iter()
            .map(|n| &n.case_id)
            .filter(|id| !duplicate_b_ids.contains(id.as_str()))
            .cloned()
            .collect();

        let total_unique = unique_cases.len();

        DeduplicationResult {
            unique_cases,
            duplicate_pairs: case_pairs,
            total_input: n,
            total_unique,
            synonym_pairs_learned: 0,
        }
    }
}

impl Default for DedupFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl AlgovigilanceFunction for DedupFunction {
    type Input = Vec<IcsrNarrative>;
    type Output = DeduplicationResult;
    type Feedback = SynonymPair;

    fn process(&self, input: &Self::Input) -> Result<Self::Output> {
        Ok(self.deduplicate_batch(input))
    }

    fn learn(&mut self, feedback: &Self::Feedback) -> Result<()> {
        // Add synonym pair for future boosting
        self.synonyms
            .push((feedback.term_a.clone(), feedback.term_b.clone()));
        Ok(())
    }

    fn decay(&mut self, _elapsed_days: f64) -> Result<DecayReport> {
        // Synonyms don't decay in-memory (store handles persistence decay)
        Ok(DecayReport {
            function_name: "dedup".to_string(),
            items_decayed: 0,
            items_below_threshold: 0,
            min_confidence: None,
            max_confidence: None,
        })
    }

    fn name(&self) -> &'static str {
        "icsr_deduplication"
    }

    fn t1_grounding(&self) -> nexcore_lex_primitiva::LexPrimitiva {
        nexcore_lex_primitiva::LexPrimitiva::Mapping
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_narrative(id: &str, text: &str) -> IcsrNarrative {
        IcsrNarrative {
            case_id: CaseId::new(id),
            narrative_text: text.to_string(),
            report_date: None,
            drug_names: Vec::new(),
            event_terms: Vec::new(),
        }
    }

    #[test]
    fn empty_batch() {
        let func = DedupFunction::new();
        let result = func.process(&vec![]).expect("process");
        assert_eq!(result.total_input, 0);
        assert_eq!(result.total_unique, 0);
    }

    #[test]
    fn single_case() {
        let func = DedupFunction::new();
        let cases = vec![make_narrative("C1", "chest pain after aspirin")];
        let result = func.process(&cases).expect("process");
        assert_eq!(result.total_input, 1);
        assert_eq!(result.total_unique, 1);
        assert!(result.duplicate_pairs.is_empty());
    }

    #[test]
    fn known_duplicates() {
        let func = DedupFunction::with_config(DedupConfig {
            similarity_threshold: 0.5,
            ..DedupConfig::default()
        });
        let cases = vec![
            make_narrative("C1", "chest pain after taking aspirin"),
            make_narrative("C2", "pain in chest following aspirin use"),
        ];
        let result = func.process(&cases).expect("process");
        assert!(!result.duplicate_pairs.is_empty());
        assert!(result.total_unique < result.total_input);
    }

    #[test]
    fn known_unique() {
        let func = DedupFunction::new();
        let cases = vec![
            make_narrative("C1", "severe headache and dizziness"),
            make_narrative("C2", "skin rash on arms and legs"),
        ];
        let result = func.process(&cases).expect("process");
        assert!(result.duplicate_pairs.is_empty());
        assert_eq!(result.total_unique, 2);
    }

    #[test]
    fn synonym_learning() {
        let mut func = DedupFunction::new();
        func.learn(
            &(SynonymPair {
                term_a: "headache".to_string(),
                term_b: "cephalgia".to_string(),
            }),
        )
        .expect("learn");
        assert_eq!(func.synonyms.len(), 1);
    }

    #[test]
    fn threshold_boundary() {
        // At exact threshold boundary
        let func = DedupFunction::with_config(DedupConfig {
            similarity_threshold: 1.0, // Only exact matches
            ..DedupConfig::default()
        });
        let cases = vec![
            make_narrative("C1", "nausea vomiting"),
            make_narrative("C2", "nausea vomiting"), // exact same
        ];
        let result = func.process(&cases).expect("process");
        assert_eq!(result.duplicate_pairs.len(), 1);
    }

    #[test]
    fn t1_grounding_is_mapping() {
        let func = DedupFunction::new();
        assert_eq!(
            func.t1_grounding(),
            nexcore_lex_primitiva::LexPrimitiva::Mapping
        );
    }
}
