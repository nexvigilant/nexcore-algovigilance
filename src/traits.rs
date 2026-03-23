//! Core trait for algovigilance functions
//!
//! Every function implements process/learn/decay with T1 grounding.

use nexcore_lex_primitiva::LexPrimitiva;

use crate::error::Result;
use crate::types::DecayReport;

/// Core trait for all algovigilance functions
///
/// Tier: T2-C (cross-domain composite trait)
/// Grounds to: T1 State (ς), Causality (→), and Persistence (π)
pub trait AlgovigilanceFunction: Send + Sync {
    /// Input data type
    type Input;
    /// Output result type
    type Output;
    /// Feedback for learning
    type Feedback;

    /// Process input and produce output
    fn process(&self, input: &Self::Input) -> Result<Self::Output>;

    /// Learn from feedback (adjust internal state)
    fn learn(&mut self, feedback: &Self::Feedback) -> Result<()>;

    /// Apply time-based decay to internal state
    fn decay(&mut self, elapsed_days: f64) -> Result<DecayReport>;

    /// Function name for logging/persistence
    fn name(&self) -> &'static str;

    /// T1 primitive grounding
    fn t1_grounding(&self) -> LexPrimitiva;
}
