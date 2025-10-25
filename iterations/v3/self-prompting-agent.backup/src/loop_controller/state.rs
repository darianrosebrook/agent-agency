//! Execution state management for the self-prompting loop controller

use std::cell::RefCell;
use tokio::sync::mpsc;
use super::types::{ExecutionState, SelfPromptingEvent};

/// Manages the execution state and user intervention capabilities
#[derive(Debug)]
pub struct ExecutionStateManager {
    /// Current execution state
    state: RefCell<ExecutionState>,
    /// Event sender for broadcasting state changes
    event_sender: Option<mpsc::UnboundedSender<SelfPromptingEvent>>,
    /// Injected guidance from user interventions
    injected_guidance: RefCell<Vec<String>>,
}

impl ExecutionStateManager {
    /// Create a new execution state manager
    pub fn new(event_sender: Option<mpsc::UnboundedSender<SelfPromptingEvent>>) -> Self {
        Self {
            state: RefCell::new(ExecutionState::Running),
            event_sender,
            injected_guidance: RefCell::new(Vec::new()),
        }
    }

    /// Get the current execution state
    pub fn current_state(&self) -> ExecutionState {
        *self.state.borrow()
    }

    /// Set the execution state and emit event if changed
    pub fn set_state(&self, new_state: ExecutionState) {
        let old_state = *self.state.borrow();
        if old_state != new_state {
            *self.state.borrow_mut() = new_state.clone();

            // Emit event for state change
            if let Some(sender) = &self.event_sender {
                let event = match new_state {
                    ExecutionState::Paused => SelfPromptingEvent::TaskPaused {
                        reason: "User intervention requested".to_string(),
                        timestamp: chrono::Utc::now(),
                    },
                    ExecutionState::Running => SelfPromptingEvent::TaskResumed {
                        timestamp: chrono::Utc::now(),
                    },
                    ExecutionState::Aborted => {
                        // Aborted events are handled by the caller
                        return;
                    }
                };

                let _ = sender.send(event);
            }
        }
    }

    /// Pause execution
    pub fn pause(&self) {
        self.set_state(ExecutionState::Paused);
    }

    /// Resume execution
    pub fn resume(&self) {
        self.set_state(ExecutionState::Running);
    }

    /// Abort execution
    pub fn abort(&self) {
        self.set_state(ExecutionState::Aborted);
    }

    /// Check if execution should continue
    pub fn should_continue(&self) -> bool {
        matches!(self.current_state(), ExecutionState::Running)
    }

    /// Check if execution is paused
    pub fn is_paused(&self) -> bool {
        matches!(self.current_state(), ExecutionState::Paused)
    }

    /// Check if execution is aborted
    pub fn is_aborted(&self) -> bool {
        matches!(self.current_state(), ExecutionState::Aborted)
    }

    /// Inject guidance for future iterations
    pub fn inject_guidance(&self, guidance: String) {
        self.injected_guidance.borrow_mut().push(guidance);
    }

    /// Get all injected guidance and clear the list
    pub fn take_injected_guidance(&self) -> Vec<String> {
        self.injected_guidance.borrow_mut().drain(..).collect()
    }

    /// Get injected guidance without clearing
    pub fn get_injected_guidance(&self) -> Vec<String> {
        self.injected_guidance.borrow().clone()
    }

    /// Override the verdict with user-provided guidance
    pub fn override_verdict(&self, new_verdict: String, reason: String) {
        info!("User verdict override: {} (reason: {})", new_verdict, reason);
        self.inject_guidance(format!("User override: {} - {}", new_verdict, reason));
    }

    /// Modify a parameter with user intervention
    pub fn modify_parameter(&self, parameter: String, value: String) {
        info!("Parameter modification: {} = {}", parameter, value);
        self.inject_guidance(format!("Parameter override: {} = {}", parameter, value));
    }
}
