//! Rule Engine for Deterministic Operator Selection
//!
//! Implements INV-CORE-04: Deterministic Selection - Given the same state,
//! the same operator must be selected.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use v4_types::task::TaskRequest;
use v4_types::OperatorType;

/// A rule for selecting operators based on conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionRule {
    /// Rule identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Priority (higher = checked first)
    pub priority: u32,
    /// Conditions that must be met
    pub conditions: Vec<RuleCondition>,
    /// Operators to propose if conditions match
    pub operators: Vec<OperatorType>,
    /// Whether this rule is enabled
    pub enabled: bool,
}

impl SelectionRule {
    /// Create a new rule
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            priority: 0,
            conditions: Vec::new(),
            operators: Vec::new(),
            enabled: true,
        }
    }

    /// Set priority
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// Add a condition
    pub fn with_condition(mut self, condition: RuleCondition) -> Self {
        self.conditions.push(condition);
        self
    }

    /// Add an operator
    pub fn with_operator(mut self, operator: OperatorType) -> Self {
        self.operators.push(operator);
        self
    }

    /// Check if all conditions are met
    pub fn matches(&self, context: &RuleContext) -> bool {
        if !self.enabled {
            return false;
        }
        self.conditions.iter().all(|c| c.evaluate(context))
    }
}

/// A condition for rule matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleCondition {
    /// Task title contains string
    TitleContains(String),
    /// Task description contains string
    DescriptionContains(String),
    /// Task has specific constraint
    HasConstraint(ConstraintType),
    /// Task priority matches
    PriorityIs(v4_types::task::TaskPriority),
    /// Environment matches
    EnvironmentIs(v4_types::task::Environment),
    /// Risk tier is at most
    RiskTierAtMost(u8),
    /// Risk tier is at least
    RiskTierAtLeast(u8),
    /// All of these conditions must match
    All(Vec<RuleCondition>),
    /// Any of these conditions must match
    Any(Vec<RuleCondition>),
    /// Negates a condition
    Not(Box<RuleCondition>),
    /// Always true
    Always,
}

impl RuleCondition {
    /// Evaluate the condition against a context
    pub fn evaluate(&self, context: &RuleContext) -> bool {
        match self {
            Self::TitleContains(s) => context
                .task_title
                .as_ref()
                .map(|t| t.to_lowercase().contains(&s.to_lowercase()))
                .unwrap_or(false),
            Self::DescriptionContains(s) => context
                .task_description
                .as_ref()
                .map(|d| d.to_lowercase().contains(&s.to_lowercase()))
                .unwrap_or(false),
            Self::HasConstraint(c) => context.constraints.contains(c),
            Self::PriorityIs(p) => context.priority.as_ref() == Some(p),
            Self::EnvironmentIs(e) => context.environment.as_ref() == Some(e),
            Self::RiskTierAtMost(tier) => context.risk_tier.map(|r| r <= *tier).unwrap_or(false),
            Self::RiskTierAtLeast(tier) => context.risk_tier.map(|r| r >= *tier).unwrap_or(false),
            Self::All(conditions) => conditions.iter().all(|c| c.evaluate(context)),
            Self::Any(conditions) => conditions.iter().any(|c| c.evaluate(context)),
            Self::Not(condition) => !condition.evaluate(context),
            Self::Always => true,
        }
    }
}

/// Types of constraints that can be checked
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstraintType {
    /// Has max duration set
    MaxDuration,
    /// Has max iterations set
    MaxIterations,
    /// Has max files set
    MaxFiles,
    /// Has path restrictions
    PathRestrictions,
    /// Allows breaking changes
    AllowsBreakingChanges,
}

/// Context for rule evaluation
#[derive(Debug, Clone, Default)]
pub struct RuleContext {
    /// Task title
    pub task_title: Option<String>,
    /// Task description
    pub task_description: Option<String>,
    /// Task priority
    pub priority: Option<v4_types::task::TaskPriority>,
    /// Environment
    pub environment: Option<v4_types::task::Environment>,
    /// Risk tier
    pub risk_tier: Option<u8>,
    /// Active constraints
    pub constraints: Vec<ConstraintType>,
    /// Additional metadata (sorted for determinism)
    pub metadata: BTreeMap<String, String>,
}

impl RuleContext {
    /// Create a new context
    pub fn new() -> Self {
        Self::default()
    }

    /// Create context from a task request
    pub fn from_task(task: &TaskRequest) -> Self {
        let mut ctx = Self {
            task_title: Some(task.title.clone()),
            task_description: Some(task.description.clone()),
            priority: Some(task.priority),
            environment: Some(task.environment),
            risk_tier: None,
            constraints: Vec::new(),
            metadata: BTreeMap::new(),
        };

        // Extract constraints
        if task.constraints.max_duration_seconds.is_some() {
            ctx.constraints.push(ConstraintType::MaxDuration);
        }
        if task.constraints.max_iterations.is_some() {
            ctx.constraints.push(ConstraintType::MaxIterations);
        }
        if task.constraints.max_files.is_some() {
            ctx.constraints.push(ConstraintType::MaxFiles);
        }
        if !task.constraints.blocked_paths.is_empty() {
            ctx.constraints.push(ConstraintType::PathRestrictions);
        }
        if task.constraints.allow_breaking_changes {
            ctx.constraints.push(ConstraintType::AllowsBreakingChanges);
        }

        ctx
    }

    /// Compute a deterministic hash of the context
    pub fn deterministic_hash(&self) -> String {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();

        // Hash in a deterministic order
        if let Some(ref title) = self.task_title {
            hasher.update(b"title:");
            hasher.update(title.as_bytes());
        }
        if let Some(ref desc) = self.task_description {
            hasher.update(b"description:");
            hasher.update(desc.as_bytes());
        }
        if let Some(ref pri) = self.priority {
            hasher.update(b"priority:");
            hasher.update(format!("{:?}", pri).as_bytes());
        }
        if let Some(ref env) = self.environment {
            hasher.update(b"environment:");
            hasher.update(format!("{:?}", env).as_bytes());
        }
        if let Some(tier) = self.risk_tier {
            hasher.update(b"risk_tier:");
            hasher.update(&[tier]);
        }

        // Constraints are already sorted by enum order
        for c in &self.constraints {
            hasher.update(b"constraint:");
            hasher.update(format!("{:?}", c).as_bytes());
        }

        // BTreeMap is sorted
        for (k, v) in &self.metadata {
            hasher.update(b"meta:");
            hasher.update(k.as_bytes());
            hasher.update(b"=");
            hasher.update(v.as_bytes());
        }

        hex::encode(hasher.finalize())
    }
}

/// Rule engine for deterministic operator selection
///
/// This engine guarantees INV-CORE-04 by:
/// 1. Sorting rules by priority (descending)
/// 2. Using deterministic hash of context for caching
/// 3. Applying first matching rule
pub struct RuleEngine {
    /// Rules sorted by priority (descending)
    rules: Vec<SelectionRule>,
    /// Cache of context hash -> result
    cache: BTreeMap<String, Vec<OperatorType>>,
    /// Whether to use caching
    caching_enabled: bool,
}

impl RuleEngine {
    /// Create a new rule engine
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            cache: BTreeMap::new(),
            caching_enabled: true,
        }
    }

    /// Disable caching
    pub fn without_cache(mut self) -> Self {
        self.caching_enabled = false;
        self
    }

    /// Add a rule to the engine
    pub fn add_rule(&mut self, rule: SelectionRule) {
        self.rules.push(rule);
        // Sort by priority descending
        self.rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        // Invalidate cache when rules change
        self.cache.clear();
    }

    /// Get all rules
    pub fn rules(&self) -> &[SelectionRule] {
        &self.rules
    }

    /// Select operators for a context
    ///
    /// Returns the operators from the first matching rule, or an empty
    /// list if no rules match.
    pub fn select(&mut self, context: &RuleContext) -> RuleEngineResult {
        let context_hash = context.deterministic_hash();

        // Check cache first
        if self.caching_enabled {
            if let Some(cached) = self.cache.get(&context_hash) {
                return RuleEngineResult {
                    operators: cached.clone(),
                    matched_rule: None, // Can't recover rule from cache
                    context_hash: context_hash.clone(),
                    from_cache: true,
                };
            }
        }

        // Find first matching rule
        for rule in &self.rules {
            if rule.matches(context) {
                let operators = rule.operators.clone();

                // Cache the result
                if self.caching_enabled {
                    self.cache.insert(context_hash.clone(), operators.clone());
                }

                return RuleEngineResult {
                    operators,
                    matched_rule: Some(rule.id.clone()),
                    context_hash,
                    from_cache: false,
                };
            }
        }

        // No match
        RuleEngineResult {
            operators: Vec::new(),
            matched_rule: None,
            context_hash,
            from_cache: false,
        }
    }

    /// Clear the cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }

    /// Create a rule engine with default rules
    pub fn with_defaults() -> Self {
        use v4_types::operators::{ControlOp, SeekOp};

        let mut engine = Self::new();

        // Rule: File reading tasks
        engine.add_rule(
            SelectionRule::new("read-file", "File Reading Task")
                .with_priority(100)
                .with_condition(RuleCondition::Any(vec![
                    RuleCondition::TitleContains("read".to_string()),
                    RuleCondition::DescriptionContains("read file".to_string()),
                ]))
                .with_operator(OperatorType::Seek(SeekOp::ReadFile {
                    path: "${file_path}".to_string(),
                })),
        );

        // Rule: Search tasks
        engine.add_rule(
            SelectionRule::new("search-code", "Code Search Task")
                .with_priority(90)
                .with_condition(RuleCondition::Any(vec![
                    RuleCondition::TitleContains("search".to_string()),
                    RuleCondition::TitleContains("find".to_string()),
                    RuleCondition::DescriptionContains("search for".to_string()),
                ]))
                .with_operator(OperatorType::Seek(SeekOp::SearchCode {
                    pattern: "${pattern}".to_string(),
                    path: None,
                })),
        );

        // Rule: High risk tasks need termination
        engine.add_rule(
            SelectionRule::new("high-risk-terminate", "High Risk Termination")
                .with_priority(200) // Higher priority
                .with_condition(RuleCondition::RiskTierAtLeast(4))
                .with_operator(OperatorType::Control(ControlOp::Terminate {
                    reason: "High risk task requires manual review".to_string(),
                })),
        );

        // Default fallback rule
        engine.add_rule(
            SelectionRule::new("default", "Default Rule")
                .with_priority(0)
                .with_condition(RuleCondition::Always)
                .with_operator(OperatorType::Seek(SeekOp::ListDirectory {
                    path: ".".to_string(),
                })),
        );

        engine
    }
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of rule engine selection
#[derive(Debug, Clone)]
pub struct RuleEngineResult {
    /// Selected operators
    pub operators: Vec<OperatorType>,
    /// ID of the matched rule (if any)
    pub matched_rule: Option<String>,
    /// Hash of the context (for verification)
    pub context_hash: String,
    /// Whether result came from cache
    pub from_cache: bool,
}

impl RuleEngineResult {
    /// Check if any operators were selected
    pub fn has_operators(&self) -> bool {
        !self.operators.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use v4_types::operators::SeekOp;

    #[test]
    fn test_rule_creation() {
        let rule = SelectionRule::new("test", "Test Rule")
            .with_priority(10)
            .with_condition(RuleCondition::TitleContains("test".to_string()))
            .with_operator(OperatorType::Seek(SeekOp::ReadFile {
                path: "test.rs".to_string(),
            }));

        assert_eq!(rule.id, "test");
        assert_eq!(rule.priority, 10);
        assert_eq!(rule.conditions.len(), 1);
        assert_eq!(rule.operators.len(), 1);
    }

    #[test]
    fn test_condition_evaluation() {
        let mut ctx = RuleContext::new();
        ctx.task_title = Some("Read the file".to_string());
        ctx.task_description = Some("Please read the configuration file".to_string());

        assert!(RuleCondition::TitleContains("read".to_string()).evaluate(&ctx));
        assert!(RuleCondition::TitleContains("READ".to_string()).evaluate(&ctx)); // Case insensitive
        assert!(!RuleCondition::TitleContains("write".to_string()).evaluate(&ctx));

        assert!(RuleCondition::DescriptionContains("configuration".to_string()).evaluate(&ctx));
    }

    #[test]
    fn test_composite_conditions() {
        let mut ctx = RuleContext::new();
        ctx.task_title = Some("Read and write".to_string());

        let all = RuleCondition::All(vec![
            RuleCondition::TitleContains("read".to_string()),
            RuleCondition::TitleContains("write".to_string()),
        ]);
        assert!(all.evaluate(&ctx));

        let any = RuleCondition::Any(vec![
            RuleCondition::TitleContains("read".to_string()),
            RuleCondition::TitleContains("delete".to_string()),
        ]);
        assert!(any.evaluate(&ctx));

        let not = RuleCondition::Not(Box::new(RuleCondition::TitleContains("delete".to_string())));
        assert!(not.evaluate(&ctx));
    }

    #[test]
    fn test_rule_engine_selection() {
        let mut engine = RuleEngine::new();

        engine.add_rule(
            SelectionRule::new("high-priority", "High Priority")
                .with_priority(100)
                .with_condition(RuleCondition::TitleContains("urgent".to_string()))
                .with_operator(OperatorType::Seek(SeekOp::ReadFile {
                    path: "urgent.rs".to_string(),
                })),
        );

        engine.add_rule(
            SelectionRule::new("low-priority", "Low Priority")
                .with_priority(10)
                .with_condition(RuleCondition::Always)
                .with_operator(OperatorType::Seek(SeekOp::ListDirectory {
                    path: ".".to_string(),
                })),
        );

        let mut ctx = RuleContext::new();
        ctx.task_title = Some("urgent task".to_string());

        let result = engine.select(&ctx);
        assert_eq!(result.matched_rule, Some("high-priority".to_string()));
        assert!(!result.from_cache);

        // Second call should use cache
        let result2 = engine.select(&ctx);
        assert!(result2.from_cache);
    }

    #[test]
    fn test_deterministic_hash() {
        let mut ctx1 = RuleContext::new();
        ctx1.task_title = Some("Test task".to_string());
        ctx1.task_description = Some("Description".to_string());

        let mut ctx2 = RuleContext::new();
        ctx2.task_title = Some("Test task".to_string());
        ctx2.task_description = Some("Description".to_string());

        assert_eq!(ctx1.deterministic_hash(), ctx2.deterministic_hash());

        // Different context should have different hash
        let mut ctx3 = RuleContext::new();
        ctx3.task_title = Some("Different task".to_string());
        assert_ne!(ctx1.deterministic_hash(), ctx3.deterministic_hash());
    }

    #[test]
    fn test_rule_priority_ordering() {
        let mut engine = RuleEngine::new();

        engine.add_rule(SelectionRule::new("low", "Low").with_priority(10));
        engine.add_rule(SelectionRule::new("high", "High").with_priority(100));
        engine.add_rule(SelectionRule::new("medium", "Medium").with_priority(50));

        assert_eq!(engine.rules[0].id, "high");
        assert_eq!(engine.rules[1].id, "medium");
        assert_eq!(engine.rules[2].id, "low");
    }

    #[test]
    fn test_default_rules() {
        let mut engine = RuleEngine::with_defaults();

        let mut ctx = RuleContext::new();
        ctx.task_title = Some("read the config file".to_string());

        let result = engine.select(&ctx);
        assert!(result.has_operators());
    }
}
