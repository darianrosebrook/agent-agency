// Quick compilation test for frontier
use std::collections::BinaryHeap;
use std::collections::HashSet;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use orchestration::frontier::{Frontier, FrontierConfig, TaskEntry};
use orchestration::planning::types::{Task, TaskType};

#[test]
fn test_frontier_compilation() {
    let config = FrontierConfig::default();
    let frontier = Frontier::with_config(config);

    let task = Task::new("test".to_string(), TaskType::Feature);
    frontier.push(task, "parent1").unwrap();

    assert!(!frontier.is_empty());
}
