//! Load testing infrastructure
//!
//! Provides load testing capabilities using K6 for performance validation
//! and stress testing of the agent system.

use std::process::Command;
use std::path::Path;

/// Load test runner for executing K6 performance tests
pub struct LoadTestRunner {
    k6_script_path: String,
}

impl LoadTestRunner {
    /// Create a new load test runner
    pub fn new(script_name: &str) -> Self {
        Self {
            k6_script_path: format!("src/load_testing/{}.js", script_name),
        }
    }

    /// Run the multimodal RAG load test
    pub fn run_multimodal_rag_test(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.run_k6_script("k6-multimodal-rag-test")
    }

    /// Run a K6 script by name
    pub fn run_k6_script(&self, script_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let script_path = Path::new("src").join("load_testing").join(format!("{}.js", script_name));

        if !script_path.exists() {
            return Err(format!("K6 script not found: {:?}", script_path).into());
        }

        println!("Running K6 load test: {}", script_name);

        let output = Command::new("k6")
            .arg("run")
            .arg(script_path)
            .output()?;

        if output.status.success() {
            println!("Load test completed successfully");
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Load test failed: {}", stderr).into())
        }
    }

    /// Get the path to the K6 script
    pub fn script_path(&self) -> &str {
        &self.k6_script_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_test_runner_creation() {
        let runner = LoadTestRunner::new("test-script");
        assert_eq!(runner.script_path(), "src/load_testing/test-script.js");
    }
}
