//! Core type definitions for the self-prompting agent system

/// Simple evaluation report
#[derive(Debug, Clone)]
pub struct EvalReport {
    pub score: f64,
    pub status: EvalStatus,
}

/// Evaluation status
#[derive(Debug, Clone, PartialEq)]
pub enum EvalStatus {
    Pass,
    Fail,
    Partial,
}
