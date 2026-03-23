//! Narrative tokenizer with medical stopword removal
//!
//! Tokenizes ICSR narratives by lowercasing, splitting on whitespace/punctuation,
//! and removing common PV stopwords. Similarity computed via Jaccard index.
//!
//! Tier: T2-C (cross-domain composite — tokenization + stopword filtering)
//! Grounds to: T1::Mapping (text → token set)

use std::collections::HashSet;

use crate::types::Similarity;

/// Medical/PV stopwords removed during tokenization
const MEDICAL_STOPWORDS: &[&str] = &[
    // Common PV narrative filler
    "patient",
    "reported",
    "reports",
    "experienced",
    "developed",
    "after",
    "before",
    "during",
    "while",
    "following",
    "taking",
    "receiving",
    "using",
    "administered",
    // Dosage forms and units
    "mg",
    "ml",
    "tablet",
    "tablets",
    "capsule",
    "capsules",
    "dose",
    "doses",
    "daily",
    "twice",
    "once",
    "oral",
    "iv",
    "im",
    "sc",
    "topical",
    // General english stopwords
    "the",
    "a",
    "an",
    "is",
    "was",
    "were",
    "are",
    "been",
    "be",
    "have",
    "has",
    "had",
    "do",
    "does",
    "did",
    "will",
    "would",
    "could",
    "should",
    "may",
    "might",
    "and",
    "or",
    "but",
    "if",
    "then",
    "than",
    "of",
    "in",
    "on",
    "at",
    "to",
    "for",
    "with",
    "by",
    "from",
    "that",
    "this",
    "these",
    "those",
    "it",
    "its",
    "not",
    "no",
    "nor",
    "very",
    "also",
];

/// Tokenize a narrative into a set of normalized tokens
///
/// Lowercases, splits on whitespace and punctuation, removes stopwords.
#[must_use]
pub fn tokenize_narrative(text: &str) -> HashSet<String> {
    let stopwords: HashSet<&str> = MEDICAL_STOPWORDS.iter().copied().collect();

    text.to_lowercase()
        .split(|c: char| c.is_whitespace() || c.is_ascii_punctuation())
        .filter(|t| !t.is_empty() && t.len() > 1)
        .filter(|t| !stopwords.contains(t))
        .map(String::from)
        .collect()
}

/// Compute Jaccard similarity between two narrative texts
///
/// Returns |intersection| / |union| of tokenized sets.
#[must_use]
pub fn narrative_similarity(a: &str, b: &str) -> Similarity {
    let set_a = tokenize_narrative(a);
    let set_b = tokenize_narrative(b);

    if set_a.is_empty() && set_b.is_empty() {
        return Similarity::new(1.0);
    }
    if set_a.is_empty() || set_b.is_empty() {
        return Similarity::new(0.0);
    }

    let intersection = set_a.intersection(&set_b).count();
    let union = set_a.union(&set_b).count();

    if union == 0 {
        Similarity::new(0.0)
    } else {
        Similarity::new(intersection as f64 / union as f64)
    }
}

/// Compute similarity with learned synonym boosting
///
/// Replaces known synonym terms before computing Jaccard.
#[must_use]
pub fn narrative_similarity_with_synonyms(
    a: &str,
    b: &str,
    synonyms: &[(String, String)],
) -> Similarity {
    let stopwords: HashSet<&str> = MEDICAL_STOPWORDS.iter().copied().collect();

    let normalize = |text: &str| -> HashSet<String> {
        text.to_lowercase()
            .split(|c: char| c.is_whitespace() || c.is_ascii_punctuation())
            .filter(|t| !t.is_empty() && t.len() > 1)
            .filter(|t| !stopwords.contains(t))
            .map(|t| {
                // Replace synonyms: if token matches term_b, use term_a as canonical
                for (canonical, variant) in synonyms {
                    if t == variant {
                        return canonical.clone();
                    }
                }
                t.to_string()
            })
            .collect()
    };

    let set_a = normalize(a);
    let set_b = normalize(b);

    if set_a.is_empty() && set_b.is_empty() {
        return Similarity::new(1.0);
    }
    if set_a.is_empty() || set_b.is_empty() {
        return Similarity::new(0.0);
    }

    let intersection = set_a.intersection(&set_b).count();
    let union = set_a.union(&set_b).count();

    if union == 0 {
        Similarity::new(0.0)
    } else {
        Similarity::new(intersection as f64 / union as f64)
    }
}

/// Compute similarity with POM registry + optional learned synonym boosting
///
/// Uses proof-of-meaning `SynonymRegistry` for curated baseline normalization,
/// then layers learned synonyms on top as fallback.
#[must_use]
pub fn narrative_similarity_with_registry(
    a: &str,
    b: &str,
    registry: &nexcore_proof_of_meaning::synonymy::SynonymRegistry,
    learned_synonyms: &[(String, String)],
) -> Similarity {
    let stopwords: HashSet<&str> = MEDICAL_STOPWORDS.iter().copied().collect();

    let normalize = |text: &str| -> HashSet<String> {
        text.to_lowercase()
            .split(|c: char| c.is_whitespace() || c.is_ascii_punctuation())
            .filter(|t| !t.is_empty() && t.len() > 1)
            .filter(|t| !stopwords.contains(t))
            .map(|t| {
                // 1. Check POM registry for curated canonical form
                if let Some((canonical, _, _)) = registry.resolve(t) {
                    return canonical.to_string();
                }
                // 2. Check learned synonyms as fallback
                for (canonical, variant) in learned_synonyms {
                    if t == variant {
                        return canonical.clone();
                    }
                }
                t.to_string()
            })
            .collect()
    };

    let set_a = normalize(a);
    let set_b = normalize(b);

    if set_a.is_empty() && set_b.is_empty() {
        return Similarity::new(1.0);
    }
    if set_a.is_empty() || set_b.is_empty() {
        return Similarity::new(0.0);
    }

    let intersection = set_a.intersection(&set_b).count();
    let union = set_a.union(&set_b).count();

    if union == 0 {
        Similarity::new(0.0)
    } else {
        Similarity::new(intersection as f64 / union as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_removes_stopwords() {
        let tokens = tokenize_narrative("The patient reported nausea after taking aspirin");
        assert!(tokens.contains("nausea"));
        assert!(tokens.contains("aspirin"));
        assert!(!tokens.contains("patient"));
        assert!(!tokens.contains("reported"));
        assert!(!tokens.contains("after"));
        assert!(!tokens.contains("taking"));
        assert!(!tokens.contains("the"));
    }

    #[test]
    fn tokenize_handles_punctuation() {
        let tokens = tokenize_narrative("chest pain, dizziness; fatigue.");
        assert!(tokens.contains("chest"));
        assert!(tokens.contains("pain"));
        assert!(tokens.contains("dizziness"));
        assert!(tokens.contains("fatigue"));
    }

    #[test]
    fn identical_narratives_score_one() {
        let sim = narrative_similarity("chest pain and nausea", "chest pain and nausea");
        assert!((sim.value() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn completely_different_score_zero() {
        let sim = narrative_similarity("headache migraine", "rash dermatitis");
        assert!((sim.value() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn similar_narratives_high_score() {
        let a = "chest pain after taking aspirin";
        let b = "pain in chest following aspirin use";
        let sim = narrative_similarity(a, b);
        // Both should tokenize to roughly: {chest, pain, aspirin}
        assert!(sim.value() > 0.4);
    }

    #[test]
    fn empty_narratives_score_one() {
        let sim = narrative_similarity("", "");
        assert!((sim.value() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn one_empty_scores_zero() {
        let sim = narrative_similarity("chest pain", "");
        assert!((sim.value() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn synonym_boosting_works() {
        let a = "headache after aspirin";
        let b = "cephalgia after aspirin";
        let base = narrative_similarity(a, b);
        let boosted = narrative_similarity_with_synonyms(
            a,
            b,
            &[("headache".to_string(), "cephalgia".to_string())],
        );
        assert!(boosted.value() > base.value());
    }

    #[test]
    fn unicode_narratives() {
        let tokens = tokenize_narrative("douleur thoracique après aspirine");
        assert!(tokens.contains("douleur"));
        assert!(tokens.contains("thoracique"));
        assert!(tokens.contains("aspirine"));
    }
}
