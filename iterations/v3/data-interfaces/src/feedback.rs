//! Feedback Module
//!
//! User feedback and progress reporting utilities.

use std::io::{self, Write};

/// Progress reporter for long-running operations
pub struct ProgressReporter {
    total: usize,
    current: usize,
}

impl ProgressReporter {
    pub fn new(total: usize) -> Self {
        Self { total, current: 0 }
    }

    pub fn update(&mut self, increment: usize) {
        self.current += increment;
        self.display();
    }

    pub fn complete(&self) {
        println!(" [DONE]");
    }

    fn display(&self) {
        let percentage = (self.current as f64 / self.total as f64 * 100.0) as usize;
        print!(
            "\rProgress: {}% ({}/{})",
            percentage, self.current, self.total
        );
        io::stdout().flush().unwrap();
    }
}
