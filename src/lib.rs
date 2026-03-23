//! # NexVigilant Core — Algovigilance — Algorithmic Vigilance Functions
//!
//! MVP with two core functions:
//! - **ICSR Deduplication**: Narrative similarity + FAERS batch dedup
//! - **Signal Triage**: Exponential decay priority queue with reinforcement
//!
//! Both implement `AlgovigilanceFunction` (process/learn/decay) and persist
//! state via federated store (Brain implicit + dedicated `~/nexcore/algovigilance/`).

#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![warn(missing_docs)]
pub mod dedup;
pub mod error;
pub mod grounding;
pub mod store;
pub mod traits;
pub mod triage;
pub mod types;

pub use error::{AlgovigilanceError, Result};
pub use traits::AlgovigilanceFunction;
pub use types::{CaseId, DecayReport, HalfLife, Relevance, SignalId, Similarity};
