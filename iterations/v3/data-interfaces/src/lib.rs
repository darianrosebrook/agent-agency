//! Data Interfaces - Unified Interface Layer & User Experience
//!
//! This crate consolidates command-line interfaces, web API endpoints, and user
//! interaction patterns into a unified interface layer for the agent system.
//!
//! ## Architecture
//!
//! The data-interfaces crate provides:
//!
//! - **CLI Interface**: Command-line tools for system interaction and administration
//! - **API Endpoints**: RESTful API endpoints for programmatic access
//! - **WebSocket Support**: Real-time communication channels
//! - **Interface Contracts**: Type-safe interface definitions and data contracts
//! - **User Experience**: Consistent interaction patterns across all interfaces

// CLI interface modules
pub mod commands;
pub mod interactive;

// API interface modules
pub mod api;
pub mod endpoints;
pub mod middleware;
pub mod websocket;

// Data contract modules
pub mod contracts;
pub mod serialization;
pub mod validation;

// Service contracts for dependency injection
pub mod service_contracts;

// User experience modules
pub mod feedback;
pub mod formatting;
pub mod ux;

// External imports
use schemars::JsonSchema;

// Re-export CLI functionality
pub use commands::*;

// Re-export API functionality
pub use endpoints::*;

// Re-export WebSocket functionality
pub use websocket::*;

// Re-export contract functionality
pub use serialization::*;

/// Main data interfaces service
#[derive(Debug)]
pub struct DataInterfacesService {
    /// CLI interface handler
    cli_interface: CliInterface,

    /// API server for endpoints
    api_server: ApiServer,

    /// WebSocket manager for real-time communication
    websocket_manager: WebSocketManager,

    /// Interface contract validator
    contract_validator: ContractValidator,
}

impl DataInterfacesService {
    /// Create a new data interfaces service
    pub async fn new() -> Result<Self, InterfaceError> {
        let cli_interface =
            CliInterface::new().map_err(|e| InterfaceError::CliError(e.to_string()))?;

        let api_server = ApiServer::new().map_err(|e| InterfaceError::ApiError(e.to_string()))?;

        let websocket_manager =
            WebSocketManager::new().map_err(|e| InterfaceError::WebSocketError(e.to_string()))?;

        let contract_validator =
            ContractValidator::new().map_err(|e| InterfaceError::ContractError(e.to_string()))?;

        Ok(Self {
            cli_interface,
            api_server,
            websocket_manager,
            contract_validator,
        })
    }

    /// Initialize the interface layer with configuration
    pub async fn initialize(&mut self, config: InterfaceConfig) -> Result<(), InterfaceError> {
        // Initialize CLI interface
        self.cli_interface
            .initialize(config.cli_config)
            .await
            .map_err(|e| InterfaceError::CliError(e.to_string()))?;

        // Initialize API server
        self.api_server
            .initialize(config.api_config)
            .await
            .map_err(|e| InterfaceError::ApiError(e.to_string()))?;

        // Initialize WebSocket manager
        self.websocket_manager
            .initialize(config.websocket_config)
            .await
            .map_err(|e| InterfaceError::WebSocketError(e.to_string()))?;

        // Initialize contract validator
        self.contract_validator
            .initialize(config.contract_config)
            .await
            .map_err(|e| InterfaceError::ContractError(e.to_string()))?;

        Ok(())
    }

    /// Start all interface services
    pub async fn start(&mut self) -> Result<(), InterfaceError> {
        // Start API server first
        self.api_server
            .start()
            .await
            .map_err(|e| InterfaceError::ApiError(e.to_string()))?;

        // Start WebSocket manager
        self.websocket_manager
            .start()
            .await
            .map_err(|e| InterfaceError::WebSocketError(e.to_string()))?;

        // Start CLI interface
        self.cli_interface
            .start()
            .await
            .map_err(|e| InterfaceError::CliError(e.to_string()))?;

        Ok(())
    }

    /// Stop all interface services
    pub async fn stop(&mut self) -> Result<(), InterfaceError> {
        // Stop in reverse order
        self.cli_interface
            .stop()
            .await
            .map_err(|e| InterfaceError::CliError(e.to_string()))?;

        self.websocket_manager
            .stop()
            .await
            .map_err(|e| InterfaceError::WebSocketError(e.to_string()))?;

        self.api_server
            .stop()
            .await
            .map_err(|e| InterfaceError::ApiError(e.to_string()))?;

        Ok(())
    }

    /// Execute CLI command
    pub async fn execute_cli_command(
        &mut self,
        command: &str,
        args: &[String],
    ) -> Result<CliResponse, InterfaceError> {
        self.cli_interface
            .execute_command(command, args)
            .await
            .map_err(|e| InterfaceError::CliError(e.to_string()))
    }

    /// Handle API request
    pub async fn handle_api_request(
        &self,
        request: ApiRequest,
    ) -> Result<ApiResponse, InterfaceError> {
        self.api_server
            .handle_request(request)
            .await
            .map_err(|e| InterfaceError::ApiError(e.to_string()))
    }

    /// Validate interface contract
    pub async fn validate_contract(
        &self,
        contract: &InterfaceContract,
    ) -> Result<ValidationResult, InterfaceError> {
        self.contract_validator
            .validate_contract(contract)
            .await
            .map_err(|e| InterfaceError::ContractError(e.to_string()))
    }

    /// Get CLI interface reference
    pub fn cli_interface(&self) -> &CliInterface {
        &self.cli_interface
    }

    /// Get API server reference
    pub fn api_server(&self) -> &ApiServer {
        &self.api_server
    }

    /// Get WebSocket manager reference
    pub fn websocket_manager(&self) -> &WebSocketManager {
        &self.websocket_manager
    }
}

/// API server wrapper
pub type ApiServer = api::ApiServer;

/// WebSocket manager wrapper
pub type WebSocketManager = websocket::WebSocketManager;

/// Contract validator wrapper
pub type ContractValidator = contracts::ContractValidator;

/// API request wrapper
pub type ApiRequest = api::ApiRequest;

/// API response wrapper
pub type ApiResponse = api::ApiResponse;

/// Interface contract wrapper
pub type InterfaceContract = contracts::InterfaceContract;

/// Validation result wrapper
pub type ValidationResult = contracts::ValidationResult;

/// Interface configuration
#[derive(Debug, Clone, JsonSchema)]
pub struct InterfaceConfig {
    pub cli_config: CliConfig,
    pub api_config: ApiConfig,
    pub websocket_config: WebSocketConfig,
    pub contract_config: ContractConfig,
}

/// API configuration
#[derive(Debug, Clone, JsonSchema)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
    pub cors_origins: Vec<String>,
    pub request_timeout_seconds: u64,
}

/// WebSocket configuration
#[derive(Debug, Clone, JsonSchema)]
pub struct WebSocketConfig {
    pub max_connections: usize,
    pub heartbeat_interval_seconds: u64,
    pub message_timeout_seconds: u64,
}

/// Contract configuration
#[derive(Debug, Clone, JsonSchema)]
pub struct ContractConfig {
    pub strict_validation: bool,
    pub schema_cache_size: usize,
    pub validation_timeout_seconds: u64,
}

/// Interface errors
#[derive(Debug, thiserror::Error, JsonSchema)]
pub enum InterfaceError {
    #[error("CLI error: {0}")]
    CliError(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("WebSocket error: {0}")]
    WebSocketError(String),

    #[error("Contract error: {0}")]
    ContractError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}
