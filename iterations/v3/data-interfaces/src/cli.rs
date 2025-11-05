//! CLI Interface Module
//!
//! This module provides the command-line interface for the data-interfaces crate.
//! It handles CLI parsing, command execution, and user interaction patterns.

use crate::{CliConfig, CliResponse, InterfaceError};
use async_trait::async_trait;
use std::collections::HashMap;

/// CLI Interface trait for command execution
#[async_trait]
pub trait CliInterfaceTrait {
    /// Execute a CLI command
    async fn execute_command(
        &mut self,
        command: &str,
        args: &[String],
    ) -> Result<CliResponse, InterfaceError>;

    /// Initialize the CLI interface
    async fn initialize(&mut self, config: CliConfig) -> Result<(), InterfaceError>;

    /// Start the CLI interface
    async fn start(&mut self) -> Result<(), InterfaceError>;

    /// Stop the CLI interface
    async fn stop(&mut self) -> Result<(), InterfaceError>;
}

/// CLI Interface implementation
pub struct CliInterface {
    /// CLI configuration
    config: CliConfig,

    /// Command registry
    commands: HashMap<String, Box<dyn Fn(&[String]) -> Result<CliResponse, InterfaceError> + Send + Sync>>,

    /// Whether the interface is running
    running: bool,
}

impl std::fmt::Debug for CliInterface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CliInterface")
            .field("config", &self.config)
            .field("commands_count", &self.commands.len())
            .field("running", &self.running)
            .finish()
    }
}

impl CliInterface {
    /// Create a new CLI interface
    pub fn new() -> Result<Self, InterfaceError> {
        Ok(Self {
            config: CliConfig {
                interactive_mode: false,
                command_timeout_seconds: 30,
                max_concurrent_commands: 10,
            },
            commands: HashMap::new(),
            running: false,
        })
    }

    /// Register a command
    pub fn register_command<F>(
        &mut self,
        name: String,
        handler: F,
    ) where
        F: Fn(&[String]) -> Result<CliResponse, InterfaceError> + Send + Sync + 'static,
    {
        self.commands.insert(name, Box::new(handler));
    }

    /// Execute a CLI command
    pub async fn execute_command(
        &mut self,
        command: &str,
        args: &[String],
    ) -> Result<CliResponse, InterfaceError> {
        if let Some(handler) = self.commands.get(command) {
            handler(args)
        } else {
            Err(InterfaceError::CliError(format!("Unknown command: {}", command)))
        }
    }

    /// Initialize the CLI interface
    pub async fn initialize(&mut self, config: CliConfig) -> Result<(), InterfaceError> {
        self.config = config;
        Ok(())
    }

    /// Start the CLI interface
    pub async fn start(&mut self) -> Result<(), InterfaceError> {
        self.running = true;
        Ok(())
    }

    /// Stop the CLI interface
    pub async fn stop(&mut self) -> Result<(), InterfaceError> {
        self.running = false;
        Ok(())
    }

    /// Check if the interface is running
    pub fn is_running(&self) -> bool {
        self.running
    }
}

#[async_trait]
impl CliInterfaceTrait for CliInterface {
    async fn execute_command(
        &mut self,
        command: &str,
        args: &[String],
    ) -> Result<CliResponse, InterfaceError> {
        self.execute_command(command, args).await
    }

    async fn initialize(&mut self, config: CliConfig) -> Result<(), InterfaceError> {
        self.initialize(config).await
    }

    async fn start(&mut self) -> Result<(), InterfaceError> {
        self.start().await
    }

    async fn stop(&mut self) -> Result<(), InterfaceError> {
        self.stop().await
    }
}

