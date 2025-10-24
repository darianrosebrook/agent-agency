//! CQRS (Command Query Responsibility Segregation) implementation
//!
//! This module separates command operations (that change state) from
//! query operations (that read state) for better architectural clarity
//! and testability.

pub mod commands;
pub mod queries;

/// Command trait - operations that change system state
#[async_trait::async_trait]
pub trait Command {
    type Result;
    type Error;

    async fn execute(&self) -> Result<Self::Result, Self::Error>;
}

/// Query trait - operations that read system state
#[async_trait::async_trait]
pub trait Query {
    type Result;
    type Error;

    async fn execute(&self) -> Result<Self::Result, Self::Error>;
}

/// Command handler trait
#[async_trait::async_trait]
pub trait CommandHandler<C: Command> {
    async fn handle(&self, command: C) -> Result<C::Result, C::Error>;
}

/// Query handler trait
#[async_trait::async_trait]
pub trait QueryHandler<Q: Query> {
    async fn handle(&self, query: Q) -> Result<Q::Result, Q::Error>;
}

/// CQRS Bus for dispatching commands and queries
/// Simplified implementation for the architectural pattern
pub struct CqrsBus;

impl CqrsBus {
    pub fn new() -> Self {
        Self
    }

    /// Execute a command directly (simplified - no handler registry)
    pub async fn execute_command<C>(&self, command: C) -> Result<C::Result, C::Error>
    where
        C: Command,
    {
        // For the architectural pattern, commands execute themselves
        // In a full implementation, this would dispatch to registered handlers
        command.execute().await
    }

    /// Execute a query directly (simplified - no handler registry)
    pub async fn execute_query<Q>(&self, query: Q) -> Result<Q::Result, Q::Error>
    where
        Q: Query,
    {
        // For the architectural pattern, queries execute themselves
        // In a full implementation, this would dispatch to registered handlers
        query.execute().await
    }
}
