use nexcore_algovigilance::triage::queue::SignalQueue;
use nexcore_algovigilance::triage::types::{SignalInput, TriageConfig};
use nexcore_algovigilance::types::SignalId;
use serde_json::Value;
use std::fs;

#[test]
fn test_hexim1_research_triage_simulation() {
    // 1. SETUP: Configure the Triage Queue
    // We use a shorter half-life for research signals to reflect the fast-paced nature of scientific validation.
    let config = TriageConfig {
        half_life_days: 14.0, // Research findings have a 2-week half-life if not reinforced
        cutoff_relevance: 0.2,
        ..TriageConfig::default()
    };
    let mut queue = SignalQueue::new(&config);

    // 2. LOAD DATA
    let report_path =
        "/home/matthew/Projects/hexim1-research/Data/Validation/HEXIM1_BET_convergence_report.json";
    let report_content = fs::read_to_string(report_path).expect("Failed to read HEXIM1 report");
    let report: Value =
        serde_json::from_str(&report_content).expect("Failed to parse HEXIM1 report");

    println!("--- HEXIM1 RESEARCH TRIAGE ---");

    // 3. INITIAL INGESTION: "Baseline Hypotheses"
    // SLE Hypothesis: Initially high confidence based on GSE50772
    let sle_input = SignalInput {
        drug: "BET-Inhibitor".to_string(),
        event: "SLE-Baseline-Elevation".to_string(),
        prr: 1.18,
        ror: 1.10,
        confidence: 0.85, // Strong initial study
    };
    queue.insert(&sle_input);

    // PD Marker Hypothesis: Confirmed across studies
    let pd_marker_input = SignalInput {
        drug: "BET-Inhibitor".to_string(),
        event: "HEXIM1-Induction".to_string(),
        prr: 4.13,
        ror: 3.50,
        confidence: 0.90,
    };
    queue.insert(&pd_marker_input);

    println!("Initial State (t=0):");
    for s in queue.signals() {
        println!(
            "  - Signal: {}, Relevance: {:.2}",
            s.signal_id,
            s.current_relevance.value()
        );
    }

    // 4. SIMULATE TIME PASSAGE (t = 30 days)
    // Applying exponential decay
    let removed = queue.decay_all(30.0);
    println!(
        "\nAfter 30 days of no new data ({} signals decayed below cutoff):",
        removed
    );
    for s in queue.signals() {
        println!(
            "  - Signal: {}, Relevance: {:.2}",
            s.signal_id,
            s.current_relevance.value()
        );
    }

    // 5. REINFORCEMENT: Clinical Trials & Human Validation
    // The report lists "3 independent clinical trials" for HEXIM1 induction.
    let pd_id = SignalId::from_pair("BET-Inhibitor", "HEXIM1-Induction");
    // Each clinical trial acts as a major reinforcement.
    queue.reinforce(&pd_id, 3); // Reinforce with 3 trials

    // 6. DECAY FOR FAILED REPLICATION
    // SLE baseline failed replication in GSE122459 and GSE81622.
    // Instead of reinforcing, we let it decay further.
    let sle_id = SignalId::from_pair("BET-Inhibitor", "SLE-Baseline-Elevation");

    println!("\nAfter Reinforcement (Clinical Trials for PD Marker):");
    for s in queue.signals() {
        let label = if s.signal_id == pd_id {
            "REINFORCED"
        } else {
            "DECAYING"
        };
        println!(
            "  - Signal: {}, Relevance: {:.2} [{}]",
            s.signal_id,
            s.current_relevance.value(),
            label
        );
    }

    // 7. ASSERTIONS
    let pd_signal = queue
        .find(&pd_id)
        .expect("PD Marker signal should still be present");
    let sle_signal = queue.find(&sle_id);

    // The failed hypothesis should have been removed (decayed below 0.2)
    assert!(sle_signal.is_none());
    assert!(pd_signal.current_relevance.value() > 0.5); // Boosted by reinforcement
    assert!(pd_signal.reinforcement_count > 0);

    println!("\nFinal Triage Result:");
    println!("  Top Priority: {}", queue.top_n(1)[0].event);
    println!("  Status: SLE Baseline Hypothesis successfully purged via decay.");
}
