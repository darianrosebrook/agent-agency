//! Event handling and broadcasting for the self-prompting loop controller

use tokio::sync::mpsc;
use chrono::{DateTime, Utc};
use super::types::SelfPromptingEvent;

/// Event broadcaster for self-prompting loop events
#[derive(Debug)]
pub struct EventBroadcaster {
    /// Channel sender for broadcasting events
    sender: Option<mpsc::UnboundedSender<SelfPromptingEvent>>,
}

impl EventBroadcaster {
    /// Create a new event broadcaster
    pub fn new(sender: Option<mpsc::UnboundedSender<SelfPromptingEvent>>) -> Self {
        Self { sender }
    }

    /// Broadcast a task started event
    pub fn task_started(&self, task_id: String, description: String) {
        self.send_event(SelfPromptingEvent::TaskStarted {
            task_id,
            description,
            timestamp: Utc::now(),
        });
    }

    /// Broadcast an iteration completed event
    pub fn iteration_completed(&self, iteration: usize, artifacts_generated: usize) {
        self.send_event(SelfPromptingEvent::IterationCompleted {
            iteration,
            artifacts_generated,
            timestamp: Utc::now(),
        });
    }

    /// Broadcast a changeset generated event
    pub fn changeset_generated(&self, changeset_id: String, files_affected: usize) {
        self.send_event(SelfPromptingEvent::ChangesetGenerated {
            changeset_id,
            files_affected,
            timestamp: Utc::now(),
        });
    }

    /// Broadcast a quality gate passed event
    pub fn quality_gate_passed(&self, gate_name: String, score: f32) {
        self.send_event(SelfPromptingEvent::QualityGatePassed {
            gate_name,
            score,
            timestamp: Utc::now(),
        });
    }

    /// Broadcast a quality gate failed event
    pub fn quality_gate_failed(&self, gate_name: String, reason: String) {
        self.send_event(SelfPromptingEvent::QualityGateFailed {
            gate_name,
            reason,
            timestamp: Utc::now(),
        });
    }

    /// Broadcast a task paused event
    pub fn task_paused(&self, reason: String) {
        self.send_event(SelfPromptingEvent::TaskPaused {
            reason,
            timestamp: Utc::now(),
        });
    }

    /// Broadcast a task resumed event
    pub fn task_resumed(&self) {
        self.send_event(SelfPromptingEvent::TaskResumed {
            timestamp: Utc::now(),
        });
    }

    /// Broadcast a task completed event
    pub fn task_completed(&self, total_iterations: usize, total_time_ms: u64, final_verdict: String) {
        self.send_event(SelfPromptingEvent::TaskCompleted {
            total_iterations,
            total_time_ms,
            final_verdict,
            timestamp: Utc::now(),
        });
    }

    /// Broadcast a task failed event
    pub fn task_failed(&self, error: String, total_iterations: usize) {
        self.send_event(SelfPromptingEvent::TaskFailed {
            error,
            total_iterations,
            timestamp: Utc::now(),
        });
    }

    /// Send an event through the channel
    fn send_event(&self, event: SelfPromptingEvent) {
        if let Some(sender) = &self.sender {
            // Log the event
            match &event {
                SelfPromptingEvent::TaskStarted { description, .. } => {
                    info!("Task started: {}", description);
                }
                SelfPromptingEvent::IterationCompleted { iteration, artifacts_generated, .. } => {
                    info!("Iteration {} completed with {} artifacts", iteration, artifacts_generated);
                }
                SelfPromptingEvent::QualityGatePassed { gate_name, score, .. } => {
                    info!("Quality gate '{}' passed with score {:.2}", gate_name, score);
                }
                SelfPromptingEvent::QualityGateFailed { gate_name, reason, .. } => {
                    warn!("Quality gate '{}' failed: {}", gate_name, reason);
                }
                SelfPromptingEvent::TaskPaused { reason, .. } => {
                    info!("Task paused: {}", reason);
                }
                SelfPromptingEvent::TaskResumed { .. } => {
                    info!("Task resumed");
                }
                SelfPromptingEvent::TaskCompleted { total_iterations, final_verdict, .. } => {
                    info!("Task completed in {} iterations with verdict: {}", total_iterations, final_verdict);
                }
                SelfPromptingEvent::TaskFailed { error, total_iterations, .. } => {
                    error!("Task failed after {} iterations: {}", total_iterations, error);
                }
                _ => {
                    debug!("Event: {:?}", event);
                }
            }

            // Send the event (ignore send errors)
            let _ = sender.send(event);
        }
    }

    /// Create an event receiver for consuming events
    pub fn create_receiver(&self) -> Option<mpsc::UnboundedReceiver<SelfPromptingEvent>> {
        // Note: This is a simplified implementation. In practice, you'd want
        // a way to create multiple receivers or use a broadcast channel.
        None // For now, events are only logged
    }
}

/// Helper function to get current UTC time
fn now() -> DateTime<Utc> {
    Utc::now()
}
