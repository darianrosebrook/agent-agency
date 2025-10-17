//! Mock implementations for integration testing

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Mock database for testing
pub struct MockDatabase {
    data: Arc<RwLock<HashMap<String, serde_json::Value>>>,
}

impl MockDatabase {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn insert(&self, key: String, value: serde_json::Value) -> Result<()> {
        let mut data = self.data.write().await;
        data.insert(key, value);
        Ok(())
    }

    pub async fn get(&self, key: &str) -> Result<Option<serde_json::Value>> {
        let data = self.data.read().await;
        Ok(data.get(key).cloned())
    }

    pub async fn delete(&self, key: &str) -> Result<()> {
        let mut data = self.data.write().await;
        data.remove(key);
        Ok(())
    }

    pub async fn clear(&self) -> Result<()> {
        let mut data = self.data.write().await;
        data.clear();
        Ok(())
    }

    pub async fn count(&self) -> usize {
        let data = self.data.read().await;
        data.len()
    }
}

/// Mock Redis for testing
pub struct MockRedis {
    data: Arc<RwLock<HashMap<String, String>>>,
}

impl MockRedis {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn set(&self, key: String, value: String) -> Result<()> {
        let mut data = self.data.write().await;
        data.insert(key, value);
        Ok(())
    }

    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        let data = self.data.read().await;
        Ok(data.get(key).cloned())
    }

    pub async fn del(&self, key: &str) -> Result<()> {
        let mut data = self.data.write().await;
        data.remove(key);
        Ok(())
    }

    pub async fn flush_all(&self) -> Result<()> {
        let mut data = self.data.write().await;
        data.clear();
        Ok(())
    }
}

/// Mock HTTP client for testing
pub struct MockHttpClient {
    responses: Arc<RwLock<HashMap<String, MockResponse>>>,
}

#[derive(Debug, Clone)]
pub struct MockResponse {
    pub status: u16,
    pub body: String,
    pub headers: HashMap<String, String>,
}

impl MockHttpClient {
    pub fn new() -> Self {
        Self {
            responses: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn mock_response(&self, url: String, response: MockResponse) {
        let mut responses = self.responses.write().await;
        responses.insert(url, response);
    }

    pub async fn get(&self, url: &str) -> Result<MockResponse> {
        let responses = self.responses.read().await;
        responses
            .get(url)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No mock response for URL: {}", url))
    }

    pub async fn post(&self, url: &str, _body: &str) -> Result<MockResponse> {
        self.get(url).await
    }
}

/// Mock model provider for testing
pub struct MockModelProvider {
    responses: Arc<RwLock<HashMap<String, String>>>,
}

impl MockModelProvider {
    pub fn new() -> Self {
        Self {
            responses: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn mock_response(&self, prompt: String, response: String) {
        let mut responses = self.responses.write().await;
        responses.insert(prompt, response);
    }

    pub async fn generate(&self, prompt: &str) -> Result<String> {
        let responses = self.responses.read().await;
        responses
            .get(prompt)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No mock response for prompt: {}", prompt))
    }
}

/// Mock file system for testing
pub struct MockFileSystem {
    files: Arc<RwLock<HashMap<String, String>>>,
}

impl MockFileSystem {
    pub fn new() -> Self {
        Self {
            files: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn write_file(&self, path: String, content: String) -> Result<()> {
        let mut files = self.files.write().await;
        files.insert(path, content);
        Ok(())
    }

    pub async fn read_file(&self, path: &str) -> Result<Option<String>> {
        let files = self.files.read().await;
        Ok(files.get(path).cloned())
    }

    pub async fn delete_file(&self, path: &str) -> Result<()> {
        let mut files = self.files.write().await;
        files.remove(path);
        Ok(())
    }

    pub async fn file_exists(&self, path: &str) -> bool {
        let files = self.files.read().await;
        files.contains_key(path)
    }

    pub async fn list_files(&self) -> Vec<String> {
        let files = self.files.read().await;
        files.keys().cloned().collect()
    }
}

/// Mock event emitter for testing
pub struct MockEventEmitter {
    events: Arc<RwLock<Vec<MockEvent>>>,
}

#[derive(Debug, Clone)]
pub struct MockEvent {
    pub event_type: String,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl MockEventEmitter {
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn emit(&self, event_type: String, data: serde_json::Value) -> Result<()> {
        let mut events = self.events.write().await;
        events.push(MockEvent {
            event_type,
            data,
            timestamp: chrono::Utc::now(),
        });
        Ok(())
    }

    pub async fn get_events(&self) -> Vec<MockEvent> {
        let events = self.events.read().await;
        events.clone()
    }

    pub async fn get_events_by_type(&self, event_type: &str) -> Vec<MockEvent> {
        let events = self.events.read().await;
        events
            .iter()
            .filter(|e| e.event_type == event_type)
            .cloned()
            .collect()
    }

    pub async fn clear_events(&self) -> Result<()> {
        let mut events = self.events.write().await;
        events.clear();
        Ok(())
    }

    pub async fn event_count(&self) -> usize {
        let events = self.events.read().await;
        events.len()
    }
}

/// Mock configuration provider for testing
pub struct MockConfigProvider {
    config: Arc<RwLock<serde_json::Value>>,
}

impl MockConfigProvider {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(serde_json::json!({}))),
        }
    }

    pub async fn set_config(&self, config: serde_json::Value) -> Result<()> {
        let mut config_guard = self.config.write().await;
        *config_guard = config;
        Ok(())
    }

    pub async fn get_config(&self) -> serde_json::Value {
        let config = self.config.read().await;
        config.clone()
    }

    pub async fn get_value(&self, key: &str) -> Result<Option<serde_json::Value>> {
        let config = self.config.read().await;
        Ok(config.get(key).cloned())
    }

    pub async fn set_value(&self, key: &str, value: serde_json::Value) -> Result<()> {
        let mut config = self.config.write().await;
        config[key] = value;
        Ok(())
    }
}

/// Mock metrics collector for testing
pub struct MockMetricsCollector {
    metrics: Arc<RwLock<HashMap<String, f64>>>,
    counters: Arc<RwLock<HashMap<String, u64>>>,
}

impl MockMetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            counters: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn record_metric(&self, name: String, value: f64) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        metrics.insert(name, value);
        Ok(())
    }

    pub async fn increment_counter(&self, name: String) -> Result<()> {
        let mut counters = self.counters.write().await;
        *counters.entry(name).or_insert(0) += 1;
        Ok(())
    }

    pub async fn get_metric(&self, name: &str) -> Option<f64> {
        let metrics = self.metrics.read().await;
        metrics.get(name).copied()
    }

    pub async fn get_counter(&self, name: &str) -> Option<u64> {
        let counters = self.counters.read().await;
        counters.get(name).copied()
    }

    pub async fn get_all_metrics(&self) -> HashMap<String, f64> {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }

    pub async fn get_all_counters(&self) -> HashMap<String, u64> {
        let counters = self.counters.read().await;
        counters.clone()
    }

    pub async fn clear(&self) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        let mut counters = self.counters.write().await;
        metrics.clear();
        counters.clear();
        Ok(())
    }
}

/// Mock timer for testing
pub struct MockTimer {
    current_time: Arc<RwLock<chrono::DateTime<chrono::Utc>>>,
}

impl MockTimer {
    pub fn new() -> Self {
        Self {
            current_time: Arc::new(RwLock::new(chrono::Utc::now())),
        }
    }

    pub async fn set_time(&self, time: chrono::DateTime<chrono::Utc>) -> Result<()> {
        let mut current_time = self.current_time.write().await;
        *current_time = time;
        Ok(())
    }

    pub async fn advance_time(&self, duration: chrono::Duration) -> Result<()> {
        let mut current_time = self.current_time.write().await;
        *current_time += duration;
        Ok(())
    }

    pub async fn now(&self) -> chrono::DateTime<chrono::Utc> {
        let current_time = self.current_time.read().await;
        *current_time
    }
}

/// Mock UUID generator for testing
pub struct MockUuidGenerator {
    uuids: Arc<RwLock<Vec<Uuid>>>,
    current_index: Arc<RwLock<usize>>,
}

impl MockUuidGenerator {
    pub fn new() -> Self {
        Self {
            uuids: Arc::new(RwLock::new(Vec::new())),
            current_index: Arc::new(RwLock::new(0)),
        }
    }

    pub async fn add_uuid(&self, uuid: Uuid) -> Result<()> {
        let mut uuids = self.uuids.write().await;
        uuids.push(uuid);
        Ok(())
    }

    pub async fn generate(&self) -> Result<Uuid> {
        let uuids = self.uuids.read().await;
        let mut index = self.current_index.write().await;

        if *index >= uuids.len() {
            return Err(anyhow::anyhow!("No more UUIDs available"));
        }

        let uuid = uuids[*index];
        *index += 1;
        Ok(uuid)
    }

    pub async fn reset(&self) -> Result<()> {
        let mut index = self.current_index.write().await;
        *index = 0;
        Ok(())
    }
}

/// Mock factory for creating all mock services
pub struct MockFactory;

impl MockFactory {
    pub fn create_database() -> MockDatabase {
        MockDatabase::new()
    }

    pub fn create_redis() -> MockRedis {
        MockRedis::new()
    }

    pub fn create_http_client() -> MockHttpClient {
        MockHttpClient::new()
    }

    pub fn create_model_provider() -> MockModelProvider {
        MockModelProvider::new()
    }

    pub fn create_file_system() -> MockFileSystem {
        MockFileSystem::new()
    }

    pub fn create_event_emitter() -> MockEventEmitter {
        MockEventEmitter::new()
    }

    pub fn create_config_provider() -> MockConfigProvider {
        MockConfigProvider::new()
    }

    pub fn create_metrics_collector() -> MockMetricsCollector {
        MockMetricsCollector::new()
    }

    pub fn create_timer() -> MockTimer {
        MockTimer::new()
    }

    pub fn create_uuid_generator() -> MockUuidGenerator {
        MockUuidGenerator::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_database() {
        let db = MockDatabase::new();

        let key = "test-key".to_string();
        let value = serde_json::json!({"test": "data"});

        db.insert(key.clone(), value.clone()).await.unwrap();
        let retrieved = db.get(&key).await.unwrap();

        assert_eq!(retrieved, Some(value));
        assert_eq!(db.count().await, 1);

        db.delete(&key).await.unwrap();
        assert_eq!(db.count().await, 0);
    }

    #[tokio::test]
    async fn test_mock_redis() {
        let redis = MockRedis::new();

        let key = "test-key".to_string();
        let value = "test-value".to_string();

        redis.set(key.clone(), value.clone()).await.unwrap();
        let retrieved = redis.get(&key).await.unwrap();

        assert_eq!(retrieved, Some(value));

        redis.del(&key).await.unwrap();
        let retrieved = redis.get(&key).await.unwrap();
        assert_eq!(retrieved, None);
    }

    #[tokio::test]
    async fn test_mock_http_client() {
        let client = MockHttpClient::new();

        let url = "https://example.com/test".to_string();
        let response = MockResponse {
            status: 200,
            body: "test response".to_string(),
            headers: HashMap::new(),
        };

        client.mock_response(url.clone(), response.clone()).await;
        let retrieved = client.get(&url).await.unwrap();

        assert_eq!(retrieved.status, 200);
        assert_eq!(retrieved.body, "test response");
    }

    #[tokio::test]
    async fn test_mock_event_emitter() {
        let emitter = MockEventEmitter::new();

        let event_type = "test-event".to_string();
        let data = serde_json::json!({"test": "data"});

        emitter
            .emit(event_type.clone(), data.clone())
            .await
            .unwrap();
        let events = emitter.get_events_by_type(&event_type).await;

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, event_type);
        assert_eq!(events[0].data, data);
    }

    #[tokio::test]
    async fn test_mock_metrics_collector() {
        let collector = MockMetricsCollector::new();

        collector
            .record_metric("test-metric".to_string(), 42.0)
            .await
            .unwrap();
        collector
            .increment_counter("test-counter".to_string())
            .await
            .unwrap();

        assert_eq!(collector.get_metric("test-metric").await, Some(42.0));
        assert_eq!(collector.get_counter("test-counter").await, Some(1));
    }
}
