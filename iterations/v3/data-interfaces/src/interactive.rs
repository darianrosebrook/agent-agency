//! Interactive CLI Module
//!
//! Interactive command-line interface for real-time user interaction.

use crate::InterfaceError;

/// Interactive CLI session
#[derive(Debug)]
pub struct InteractiveSession {
    pub active: bool,
    pub current_context: String,
    pub history: Vec<String>,
}

impl InteractiveSession {
    /// Create a new interactive session
    pub fn new() -> Self {
        Self {
            active: false,
            current_context: "default".to_string(),
            history: Vec::new(),
        }
    }

    /// Start the interactive session
    pub async fn start(&mut self) -> Result<(), InterfaceError> {
        self.active = true;
        println!("Agent Agency Interactive CLI");
        println!("Type 'help' for commands, 'quit' to exit");
        Ok(())
    }

    /// Stop the interactive session
    pub async fn stop(&mut self) -> Result<(), InterfaceError> {
        self.active = false;
        println!("Goodbye!");
        Ok(())
    }

    /// Process user input
    pub async fn process_input(&mut self, input: &str) -> Result<String, InterfaceError> {
        self.history.push(input.to_string());

        match input.trim() {
            "help" => Ok("Available commands: help, status, quit".to_string()),
            "status" => Ok("System status: Running".to_string()),
            "quit" => {
                self.stop().await?;
                Ok("Session ended".to_string())
            }
            _ => Ok(format!("Unknown command: {}", input)),
        }
    }
}
