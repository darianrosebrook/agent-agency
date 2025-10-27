//! Local service management for E2E tests
//!
//! Provides lifecycle management for services required by autonomous tests:
//! - OrchestratorService: CoreML Mistral model instance for task orchestration (REAL COREM L)
//! - OllamaService: Local Ollama instance for model inference (REAL HTTP CALLS)
//! - PostgresService: PostgreSQL database for persistence (REAL DATABASE CONNECTIONS)

pub mod orchestrator;
pub mod ollama;
pub mod postgres;

pub use orchestrator::OrchestratorService;
pub use ollama::OllamaService;
pub use postgres::PostgresService;
