//! E2E test scenarios implementation
//!
//! Contains the actual test implementations for each scenario:
//! - Scenario 1: Long-horizon refactor with iterative improvement
//! - Scenario 2: Research and synthesis with data persistence
//! - Scenario 3: Code generation and mutation testing
//! - Scenario 4: Autonomous file editing with Git worktrees
//! - CAWS Governance: Constitutional authority and waiver management
//! - Self-Prompting Loops: Iterative improvement with satisficing
//! - Human Intervention: Pause/resume/cancel with real-time control
//! - Reflexive Learning: Continuous improvement through feedback
//! - Multi-Agent Coordination: Agent communication and arbitration
//! - Claim Verification: Factual accuracy and hallucination detection
//! - Performance & Scalability: Load testing and optimization validation
//! - Security & Privacy: Safe operation and compliance validation

#[cfg(feature = "full")]
pub mod scenario_1_refactor;
#[cfg(feature = "full")]
pub mod scenario_2_research;
#[cfg(feature = "full")]
pub mod scenario_3_mutation;
pub mod scenario_4_file_editing;
#[cfg(feature = "full")]
pub mod autonomous_workflow;

// CAWS Constitutional Authority tests
pub mod caws_governance;

// Self-Prompting Loop tests
#[cfg(feature = "full")]
pub mod self_prompting_loops;

// Human Intervention tests
pub mod human_intervention;

// Reflexive Learning tests
#[cfg(feature = "full")]
pub mod reflexive_learning;

// Multi-Agent Coordination tests
#[cfg(feature = "full")]
pub mod multi_agent_coordination;

// Claim Extraction & Verification tests
#[cfg(feature = "full")]
pub mod claim_verification;

// Performance & Scalability tests
pub mod performance_scalability;

// Security & Privacy tests
pub mod security_privacy;

