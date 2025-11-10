//! Event emission and handling

use tokio::sync::mpsc;
use super::core::ResearchEvent;

/// Event emitter for research events

#[derive(Debug)]
pub struct EventEmitter {
    sender: mpsc::UnboundedSender<ResearchEvent>,
}

impl EventEmitter {
    /// Create a new event emitter
    pub fn new() -> Self {
        let (sender, _receiver) = mpsc::unbounded_channel();
        Self { sender }
    }

    /// Emit an event
    pub async fn emit(&self, event: ResearchEvent) {
        let _ = self.sender.send(event);
    }
}
