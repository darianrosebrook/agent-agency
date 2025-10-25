//! Test versioning and history management

use super::types::*;
use std::collections::HashMap;

/// Test versioning manager
#[derive(Debug)]
pub struct VersioningManager {
    version_history: HashMap<String, Vec<TestVersion>>,
}

impl VersioningManager {
    pub fn new() -> Self {
        Self {
            version_history: HashMap::new(),
        }
    }

    pub fn create_version(&mut self, test_spec: &TestSpecification) -> TestVersion {
        let version = TestVersion {
            version_id: uuid::Uuid::new_v4().to_string(),
            test_spec_id: test_spec.test_id.clone(),
            version_number: self.get_next_version(&test_spec.test_id),
            created_at: chrono::Utc::now(),
            changes: vec!["Initial version".to_string()],
        };

        self.version_history
            .entry(test_spec.test_id.clone())
            .or_insert_with(Vec::new)
            .push(version.clone());

        version
    }

    fn get_next_version(&self, test_spec_id: &str) -> u32 {
        self.version_history
            .get(test_spec_id)
            .map(|versions| versions.len() as u32 + 1)
            .unwrap_or(1)
    }

    pub fn get_version_history(&self, test_spec_id: &str) -> Vec<TestVersion> {
        self.version_history
            .get(test_spec_id)
            .cloned()
            .unwrap_or_default()
    }
}

/// Test version information
#[derive(Debug, Clone)]
pub struct TestVersion {
    pub version_id: String,
    pub test_spec_id: String,
    pub version_number: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub changes: Vec<String>,
}