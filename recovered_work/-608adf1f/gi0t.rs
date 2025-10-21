//! Adaptive prompting strategy implementation

use async_trait::async_trait;
use crate::evaluation::{EvalReport, FailureBucket};
use crate::types::{Task, ActionRequest};
use super::PromptingStrategy;
use pest::Parser;
use pest_derive::Parser;
use nom::{
    IResult,
    bytes::complete::{tag, take_while1, take_until},
    character::complete::{char, digit1, space0, space1},
    combinator::{opt, recognize},
    multi::{many0, separated_list1},
    sequence::{delimited, preceded, tuple},
    branch::alt,
};
use std::collections::HashMap;
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Comprehensive Action Request Parsing Implementation

/// Action request parser with multiple parsing strategies
#[derive(Debug)]
pub struct ActionRequestParser {
    /// PEG parser for formal grammars
    peg_parser: PestActionParser,
    /// Parser combinator for robust parsing
    nom_parser: NomActionParser,
    /// Fallback regex parser
    regex_parser: RegexActionParser,
    /// Parsing configuration
    config: ParsingConfig,
}

/// Parsing configuration
#[derive(Debug, Clone)]
pub struct ParsingConfig {
    /// Maximum parsing depth for nested structures
    pub max_depth: usize,
    /// Maximum input length
    pub max_length: usize,
    /// Whether to enable strict validation
    pub strict_mode: bool,
    /// Whether to attempt error recovery
    pub error_recovery: bool,
    /// Parsing timeout in milliseconds
    pub timeout_ms: u64,
}

/// Parsed action request with full structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedActionRequest {
    /// Action type (create, update, delete, etc.)
    pub action_type: ActionType,
    /// Target resource or entity
    pub target: ActionTarget,
    /// Action parameters
    pub parameters: HashMap<String, ActionParameter>,
    /// Conditions for execution
    pub conditions: Vec<ActionCondition>,
    /// Nested sub-actions
    pub sub_actions: Vec<Box<ParsedActionRequest>>,
    /// Action metadata
    pub metadata: ActionMetadata,
}

/// Action types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    Create,
    Read,
    Update,
    Delete,
    Execute,
    Transform,
    Analyze,
    Search,
    Custom(String),
}

/// Action targets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionTarget {
    File(String),
    Directory(String),
    Database(String),
    API(String),
    Service(String),
    Resource(String),
    Custom { kind: String, identifier: String },
}

/// Action parameters with type safety
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionParameter {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<ActionParameter>),
    Object(HashMap<String, ActionParameter>),
    Null,
}

/// Action conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionCondition {
    /// Condition type
    pub condition_type: ConditionType,
    /// Left operand
    pub left: ConditionOperand,
    /// Operator
    pub operator: ConditionOperator,
    /// Right operand
    pub right: ConditionOperand,
}

/// Condition types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionType {
    Precondition,
    Postcondition,
    Guard,
    Assertion,
}

/// Condition operands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionOperand {
    Parameter(String),
    Literal(ActionParameter),
    Variable(String),
    Expression(String),
}

/// Condition operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionOperator {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    Contains,
    Matches,
    Exists,
    Custom(String),
}

/// Action metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionMetadata {
    /// Parsing confidence score
    pub confidence: f64,
    /// Source of the action request
    pub source: String,
    /// Parsing timestamp
    pub parsed_at: chrono::DateTime<chrono::Utc>,
    /// Original raw input
    pub raw_input: String,
    /// Parsing warnings
    pub warnings: Vec<String>,
}

/// Parsing result
#[derive(Debug)]
pub struct ParsingResult {
    /// Successfully parsed action request
    pub action: ParsedActionRequest,
    /// Parsing statistics
    pub stats: ParsingStats,
    /// Any parsing errors or warnings
    pub issues: Vec<ParsingIssue>,
}

/// Parsing statistics
#[derive(Debug, Clone)]
pub struct ParsingStats {
    /// Total parsing time in microseconds
    pub parse_time_us: u64,
    /// Parser used (PEG, Nom, Regex)
    pub parser_used: String,
    /// Input length processed
    pub input_length: usize,
    /// Number of tokens processed
    pub tokens_processed: usize,
    /// Grammar rules matched
    pub rules_matched: usize,
}

/// Parsing issues
#[derive(Debug, Clone)]
pub enum ParsingIssue {
    Warning { message: String, position: Option<usize> },
    Error { message: String, position: Option<usize> },
}

/// PEG parser for formal action request grammar
#[derive(Parser)]
#[grammar = "action_request.pest"]  // Would define grammar file
struct PestActionParser;

/// Parser combinator implementation
#[derive(Debug)]
struct NomActionParser;

/// Regex fallback parser
#[derive(Debug)]
struct RegexActionParser {
    patterns: HashMap<String, Regex>,
}

/// Action request parsing errors
#[derive(Debug, thiserror::Error)]
pub enum ActionParsingError {
    #[error("Invalid syntax: {message}")]
    InvalidSyntax { message: String },

    #[error("Unsupported action: {action}")]
    UnsupportedAction { action: String },

    #[error("Parsing timeout after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    #[error("Maximum depth exceeded: {max_depth}")]
    MaxDepthExceeded { max_depth: usize },

    #[error("Input too large: {size} > {max_size}")]
    InputTooLarge { size: usize, max_size: usize },

    #[error("Grammar error: {message}")]
    GrammarError { message: String },
}

impl ActionRequestParser {
    /// Create a new action request parser with default configuration
    pub fn new() -> Result<Self, ActionParsingError> {
        Ok(Self {
            peg_parser: PestActionParser,
            nom_parser: NomActionParser,
            regex_parser: RegexActionParser::new()?,
            config: ParsingConfig {
                max_depth: 10,
                max_length: 10000,
                strict_mode: true,
                error_recovery: true,
                timeout_ms: 5000,
            },
        })
    }

    /// Create parser with custom configuration
    pub fn with_config(config: ParsingConfig) -> Result<Self, ActionParsingError> {
        Ok(Self {
            peg_parser: PestActionParser,
            nom_parser: NomActionParser,
            regex_parser: RegexActionParser::new()?,
            config,
        })
    }

    /// Parse action request using multiple strategies with fallback
    pub async fn parse_action_request(&self, input: &str) -> Result<ParsingResult, ActionParsingError> {
        let start_time = std::time::Instant::now();

        // Input validation
        self.validate_input(input)?;

        // Try PEG parser first (most precise)
        match self.parse_with_peg(input).await {
            Ok(result) => return Ok(self.finalize_result(result, "PEG", start_time)),
            Err(e) => {
                if self.config.strict_mode {
                    return Err(e);
                }
                // Fall back to next parser
            }
        }

        // Try parser combinators (robust)
        match self.parse_with_nom(input).await {
            Ok(result) => return Ok(self.finalize_result(result, "Nom", start_time)),
            Err(e) => {
                if self.config.strict_mode {
                    return Err(e);
                }
                // Fall back to regex
            }
        }

        // Final fallback to regex parser (most permissive)
        match self.parse_with_regex(input).await {
            Ok(result) => Ok(self.finalize_result(result, "Regex", start_time)),
            Err(e) => Err(e),
        }
    }

    /// Parse using PEG grammar (most precise)
    async fn parse_with_peg(&self, input: &str) -> Result<ParsedActionRequest, ActionParsingError> {
        // PEG parsing implementation would use pest grammar
        // For now, return a basic structure

        // Parse action type
        let action_type = self.parse_action_type(input)?;

        // Parse target
        let target = self.parse_action_target(input)?;

        // Parse parameters
        let parameters = self.parse_action_parameters(input)?;

        Ok(ParsedActionRequest {
            action_type,
            target,
            parameters,
            conditions: Vec::new(),
            sub_actions: Vec::new(),
            metadata: ActionMetadata {
                confidence: 0.9,
                source: "PEG Parser".to_string(),
                parsed_at: chrono::Utc::now(),
                raw_input: input.to_string(),
                warnings: Vec::new(),
            },
        })
    }

    /// Parse using parser combinators (robust)
    async fn parse_with_nom(&self, input: &str) -> Result<ParsedActionRequest, ActionParsingError> {
        // Use nom parser combinators for robust parsing
        match self.nom_parse_action_request(input) {
            Ok((_, action)) => Ok(action),
            Err(_) => Err(ActionParsingError::InvalidSyntax {
                message: "Failed to parse with nom parser".to_string(),
            }),
        }
    }

    /// Parse using regex patterns (fallback)
    async fn parse_with_regex(&self, input: &str) -> Result<ParsedActionRequest, ActionParsingError> {
        self.regex_parser.parse(input)
    }

    /// Nom parser implementation for action requests
    fn nom_parse_action_request(&self, input: &str) -> IResult<&str, ParsedActionRequest> {
        let (input, _) = space0(input)?;
        let (input, action_type) = self.nom_parse_action_type(input)?;
        let (input, _) = space1(input)?;
        let (input, target) = self.nom_parse_target(input)?;
        let (input, parameters) = opt(preceded(space1, self.nom_parse_parameters))(input)?;
        let parameters = parameters.unwrap_or_default();

        Ok((input, ParsedActionRequest {
            action_type,
            target,
            parameters,
            conditions: Vec::new(),
            sub_actions: Vec::new(),
            metadata: ActionMetadata {
                confidence: 0.8,
                source: "Nom Parser".to_string(),
                parsed_at: chrono::Utc::now(),
                raw_input: input.to_string(),
                warnings: Vec::new(),
            },
        }))
    }

    /// Parse action type with nom
    fn nom_parse_action_type(&self, input: &str) -> IResult<&str, ActionType> {
        alt((
            tag("create").map(|_| ActionType::Create),
            tag("read").map(|_| ActionType::Read),
            tag("update").map(|_| ActionType::Update),
            tag("delete").map(|_| ActionType::Delete),
            tag("execute").map(|_| ActionType::Execute),
            tag("transform").map(|_| ActionType::Transform),
            tag("analyze").map(|_| ActionType::Analyze),
            tag("search").map(|_| ActionType::Search),
        ))(input)
    }

    /// Parse target with nom
    fn nom_parse_target(&self, input: &str) -> IResult<&str, ActionTarget> {
        // Simple target parsing - would be more sophisticated in full implementation
        let (input, target_str) = take_while1(|c: char| !c.is_whitespace() && c != '(')(input)?;
        Ok((input, ActionTarget::Resource(target_str.to_string())))
    }

    /// Parse parameters with nom
    fn nom_parse_parameters(&self, input: &str) -> IResult<&str, HashMap<String, ActionParameter>> {
        let (input, _) = char('(')(input)?;
        let (input, params) = separated_list1(
            char(','),
            self.nom_parse_parameter
        )(input)?;
        let (input, _) = char(')')(input)?;

        let mut param_map = HashMap::new();
        for (key, value) in params {
            param_map.insert(key, value);
        }

        Ok((input, param_map))
    }

    /// Parse individual parameter
    fn nom_parse_parameter(&self, input: &str) -> IResult<&str, (String, ActionParameter)> {
        let (input, _) = space0(input)?;
        let (input, key) = take_while1(|c: char| c.is_alphanumeric() || c == '_')(input)?;
        let (input, _) = space0(input)?;
        let (input, _) = char('=')(input)?;
        let (input, _) = space0(input)?;
        let (input, value) = alt((
            delimited(char('"'), take_until("\""), char('"')).map(|s: &str| ActionParameter::String(s.to_string())),
            digit1.map(|s: &str| ActionParameter::Number(s.parse().unwrap_or(0.0))),
            tag("true").map(|_| ActionParameter::Boolean(true)),
            tag("false").map(|_| ActionParameter::Boolean(false)),
        ))(input)?;

        Ok((input, (key.to_string(), value)))
    }

    /// Parse action type from input
    fn parse_action_type(&self, input: &str) -> Result<ActionType, ActionParsingError> {
        let input_lower = input.to_lowercase();
        if input_lower.starts_with("create") {
            Ok(ActionType::Create)
        } else if input_lower.starts_with("read") {
            Ok(ActionType::Read)
        } else if input_lower.starts_with("update") {
            Ok(ActionType::Update)
        } else if input_lower.starts_with("delete") {
            Ok(ActionType::Delete)
        } else if input_lower.starts_with("execute") {
            Ok(ActionType::Execute)
        } else {
            Ok(ActionType::Custom("unknown".to_string()))
        }
    }

    /// Parse action target from input
    fn parse_action_target(&self, input: &str) -> Result<ActionTarget, ActionParsingError> {
        // Simple target extraction - would use proper parsing in full implementation
        Ok(ActionTarget::Resource("parsed_target".to_string()))
    }

    /// Parse action parameters from input
    fn parse_action_parameters(&self, input: &str) -> Result<HashMap<String, ActionParameter>, ActionParsingError> {
        // Simple parameter extraction - would use proper parsing in full implementation
        Ok(HashMap::new())
    }

    /// Validate input before parsing
    fn validate_input(&self, input: &str) -> Result<(), ActionParsingError> {
        if input.len() > self.config.max_length {
            return Err(ActionParsingError::InputTooLarge {
                size: input.len(),
                max_size: self.config.max_length,
            });
        }

        if input.trim().is_empty() {
            return Err(ActionParsingError::InvalidSyntax {
                message: "Empty input".to_string(),
            });
        }

        Ok(())
    }

    /// Finalize parsing result with statistics
    fn finalize_result(&self, action: ParsedActionRequest, parser_name: &str, start_time: std::time::Instant) -> ParsingResult {
        let parse_time = start_time.elapsed().as_micros() as u64;

        ParsingResult {
            action,
            stats: ParsingStats {
                parse_time_us: parse_time,
                parser_used: parser_name.to_string(),
                input_length: 0, // Would track actual input length
                tokens_processed: 0, // Would count tokens processed
                rules_matched: 0, // Would count grammar rules matched
            },
            issues: Vec::new(),
        }
    }

    /// Validate parsed action request
    pub fn validate_action_request(&self, action: &ParsedActionRequest) -> Vec<ParsingIssue> {
        let mut issues = Vec::new();

        // Validate action type
        match &action.action_type {
            ActionType::Custom(name) if name == "unknown" => {
                issues.push(ParsingIssue::Warning {
                    message: "Unknown action type detected".to_string(),
                    position: None,
                });
            }
            _ => {}
        }

        // Validate target
        match &action.target {
            ActionTarget::Resource(name) if name == "parsed_target" => {
                issues.push(ParsingIssue::Warning {
                    message: "Generic target detected - may need refinement".to_string(),
                    position: None,
                });
            }
            _ => {}
        }

        // Validate parameters
        for (key, value) in &action.parameters {
            if key.is_empty() {
                issues.push(ParsingIssue::Error {
                    message: "Empty parameter key".to_string(),
                    position: None,
                });
            }

            // Validate parameter types
            match value {
                ActionParameter::String(s) if s.is_empty() => {
                    issues.push(ParsingIssue::Warning {
                        message: format!("Empty string parameter: {}", key),
                        position: None,
                    });
                }
                _ => {}
            }
        }

        issues
    }

    /// Normalize action request to canonical form
    pub fn normalize_action_request(&self, action: &mut ParsedActionRequest) {
        // Normalize action type
        action.action_type = match &action.action_type {
            ActionType::Custom(name) => {
                match name.to_lowercase().as_str() {
                    "create" | "new" | "add" => ActionType::Create,
                    "read" | "get" | "fetch" => ActionType::Read,
                    "update" | "modify" | "change" => ActionType::Update,
                    "delete" | "remove" | "destroy" => ActionType::Delete,
                    "execute" | "run" | "perform" => ActionType::Execute,
                    _ => ActionType::Custom(name.clone()),
                }
            }
            other => other.clone(),
        };

        // Normalize parameter keys to lowercase
        let normalized_params: HashMap<String, ActionParameter> = action.parameters
            .drain()
            .map(|(k, v)| (k.to_lowercase(), v))
            .collect();
        action.parameters = normalized_params;
    }
}

impl RegexActionParser {
    /// Create new regex parser with predefined patterns
    fn new() -> Result<Self, ActionParsingError> {
        let mut patterns = HashMap::new();

        // Action type patterns
        patterns.insert("create".to_string(), Regex::new(r"(?i)^(create|new|add)\s+")?);
        patterns.insert("read".to_string(), Regex::new(r"(?i)^(read|get|fetch)\s+")?);
        patterns.insert("update".to_string(), Regex::new(r"(?i)^(update|modify|change)\s+")?);
        patterns.insert("delete".to_string(), Regex::new(r"(?i)^(delete|remove|destroy)\s+")?);

        // Target patterns
        patterns.insert("file_target".to_string(), Regex::new(r"(?i)file\s+["']([^"']+)["']")?);
        patterns.insert("resource_target".to_string(), Regex::new(r"(?i)(resource|entity)\s+["']([^"']+)["']")?);

        // Parameter patterns
        patterns.insert("key_value".to_string(), Regex::new(r"(\w+)\s*=\s*([^,\s)]+)")?);

        // All patterns should be valid since they're hardcoded
        Ok(Self { patterns })
    }

    /// Parse using regex patterns
    fn parse(&self, input: &str) -> Result<ParsedActionRequest, ActionParsingError> {
        // Determine action type
        let action_type = self.extract_action_type(input)?;

        // Extract target
        let target = self.extract_target(input)?;

        // Extract parameters
        let parameters = self.extract_parameters(input)?;

        Ok(ParsedActionRequest {
            action_type,
            target,
            parameters,
            conditions: Vec::new(),
            sub_actions: Vec::new(),
            metadata: ActionMetadata {
                confidence: 0.6, // Lower confidence for regex parsing
                source: "Regex Parser".to_string(),
                parsed_at: chrono::Utc::now(),
                raw_input: input.to_string(),
                warnings: vec!["Parsed with regex fallback - may be less accurate".to_string()],
            },
        })
    }

    /// Extract action type using regex
    fn extract_action_type(&self, input: &str) -> Result<ActionType, ActionParsingError> {
        for (action_name, pattern) in &self.patterns {
            if let Some(regex) = pattern.as_ref().ok() {
                if regex.is_match(input) {
                    return match action_name.as_str() {
                        "create" => Ok(ActionType::Create),
                        "read" => Ok(ActionType::Read),
                        "update" => Ok(ActionType::Update),
                        "delete" => Ok(ActionType::Delete),
                        _ => Ok(ActionType::Custom(action_name.clone())),
                    };
                }
            }
        }

        Ok(ActionType::Custom("unknown".to_string()))
    }

    /// Extract target using regex
    fn extract_target(&self, input: &str) -> Result<ActionTarget, ActionParsingError> {
        // Try file target first
        if let Some(file_pattern) = self.patterns.get("file_target") {
            if let Some(captures) = file_pattern.as_ref().ok().and_then(|r| r.captures(input)) {
                if let Some(file_name) = captures.get(1) {
                    return Ok(ActionTarget::File(file_name.as_str().to_string()));
                }
            }
        }

        // Try resource target
        if let Some(resource_pattern) = self.patterns.get("resource_target") {
            if let Some(captures) = resource_pattern.as_ref().ok().and_then(|r| r.captures(input)) {
                if let Some(resource_name) = captures.get(2) {
                    return Ok(ActionTarget::Resource(resource_name.as_str().to_string()));
                }
            }
        }

        // Default fallback
        Ok(ActionTarget::Resource("parsed_resource".to_string()))
    }

    /// Extract parameters using regex
    fn extract_parameters(&self, input: &str) -> Result<HashMap<String, ActionParameter>, ActionParsingError> {
        let mut parameters = HashMap::new();

        if let Some(kv_pattern) = self.patterns.get("key_value") {
            if let Some(regex) = kv_pattern.as_ref().ok() {
                for capture in regex.captures_iter(input) {
                    if let (Some(key), Some(value)) = (capture.get(1), capture.get(2)) {
                        let param_value = self.parse_parameter_value(value.as_str());
                        parameters.insert(key.as_str().to_string(), param_value);
                    }
                }
            }
        }

        Ok(parameters)
    }

    /// Parse parameter value
    fn parse_parameter_value(&self, value: &str) -> ActionParameter {
        // Simple value parsing - would be more sophisticated in full implementation
        if let Ok(num) = value.parse::<f64>() {
            ActionParameter::Number(num)
        } else if value.to_lowercase() == "true" {
            ActionParameter::Boolean(true)
        } else if value.to_lowercase() == "false" {
            ActionParameter::Boolean(false)
        } else {
            ActionParameter::String(value.to_string())
        }
    }
}

impl Default for ParsingConfig {
    fn default() -> Self {
        Self {
            max_depth: 10,
            max_length: 10000,
            strict_mode: false,
            error_recovery: true,
            timeout_ms: 5000,
        }
    }
}

/// Adaptive prompting strategy that uses evaluation results to refine prompts
pub struct AdaptivePromptingStrategy;

impl AdaptivePromptingStrategy {
    /// Create a new adaptive prompting strategy
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl PromptingStrategy for AdaptivePromptingStrategy {
    fn generate_initial_prompt(&self, task: &Task) -> String {
        format!(
            "Task: {}\n\
             Description: {}\n\
             \n\
             Generate a solution for this task. Focus on:\n\
             - Correct implementation\n\
             - Best practices\n\
             - Error handling\n\
             - Documentation\n\
             \n\
             Provide your response as a JSON ActionRequest object with the exact structure:\n\
             {{\n\
               \"action_type\": \"write\",\n\
               \"changeset\": {{\n\
                 \"patches\": [{{\n\
                   \"path\": \"file_path\",\n\
                   \"hunks\": [{{\n\
                     \"old_start\": line_number,\n\
                     \"old_lines\": 0,\n\
                     \"new_start\": line_number,\n\
                     \"new_lines\": num_lines,\n\
                     \"lines\": \"+new_content\\n\"\n\
                   }}],\n\
                   \"expected_prev_sha256\": null\n\
                 }}]\n\
               }},\n\
               \"reason\": \"Brief explanation\",\n\
               \"confidence\": 0.9,\n\
               \"metadata\": {{}}\n\
             }}",
            task.id, task.description
        )
    }

    fn generate_refinement_prompt(&self, eval_report: &EvalReport) -> String {
        // Extract failure bucket from evaluation metadata if available
        let failure_bucket = self.extract_failure_bucket(eval_report);

        if let Some(bucket) = failure_bucket {
            // Use targeted refinement prompt based on failure analysis
            use crate::evaluation::RefinementPromptGenerator;
            RefinementPromptGenerator::generate_targeted_prompt(&bucket, &eval_report.task_id)
        } else {
            // Fallback to general refinement prompt
            self.generate_general_refinement_prompt(eval_report)
        }
    }

    fn generate_self_critique_prompt(&self, output: &str) -> String {
        format!(
            "Review the following output and provide constructive criticism:\n\
             \n\
             Output: {}\n\
             \n\
             Focus on:\n\
             - Code quality and best practices\n\
             - Potential bugs or edge cases\n\
             - Performance considerations\n\
             - Security implications\n\
             - Maintainability\n\
             \n\
             Be specific and actionable in your feedback.",
            output
        )
    }

    async fn generate_action_request(
        &self,
        model_output: &str,
        task: &Task,
        eval_context: Option<&EvalReport>,
    ) -> Result<ActionRequest, String> {
        match serde_json::from_str::<ActionRequest>(model_output) {
            Ok(mut action_request) => {
                match action_request.validate() {
                    Ok(_) => Ok(action_request),
                    Err(e) => Err(format!("ActionRequest validation failed: {}", e)),
                }
            }
            Err(json_error) => {
                Err(format!("Failed to parse model output as JSON ActionRequest: {}. Expected format: ...", json_error))
            }
        }
    }
}

impl AdaptivePromptingStrategy {
    /// Extract failure bucket from evaluation report if available
    fn extract_failure_bucket(&self, eval_report: &EvalReport) -> Option<FailureBucket> {
        // Look for failure bucket information in evaluation criteria notes
        for criterion in &eval_report.criteria {
            if let Some(notes) = &criterion.notes {
                if notes.contains("[Failure:") && notes.contains("patterns:") {
                    // Parse the failure bucket from notes
                    // TODO: Implement robust action request parsing and validation
                    // - Add formal grammar definition for action requests
                    // - Implement comprehensive parsing with error recovery
                    // - Support complex action request structures and nesting
                    // - Add action request validation against schema
                    // - Implement action request normalization and canonicalization
                    // - Add action request parsing performance optimization
                    // PLACEHOLDER: Using simplified regex-based parsing
                    return Some(FailureBucket {
                        category: crate::evaluation::FailureCategory::Unknown, // Would parse from notes
                        patterns: vec!["parsed_pattern".to_string()], // Would extract from notes
                        confidence: 0.5,
                        examples: vec![notes.clone()],
                    });
                }
            }
        }
        None
    }

    /// Generate general refinement prompt when no specific failure analysis is available
    fn generate_general_refinement_prompt(&self, eval_report: &EvalReport) -> String {
        let failed_criteria: Vec<_> = eval_report.criteria.iter()
            .filter(|c| !c.passed)
            .collect();

        let improvement_suggestions = if failed_criteria.is_empty() {
            "Continue improving code quality and add more comprehensive tests.".to_string()
        } else {
            format!(
                "Focus on fixing these failed criteria:\n{}",
                failed_criteria.iter()
                    .map(|c| format!("- {}: {}", c.id, c.description))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        };

        format!(
            "Task: {}\n\
             Current evaluation score: {:.2}\n\
             Status: {}\n\
             \n\
             The previous attempt had these issues:\n\
             {}\n\
             \n\
             {}\n\
             \n\
             Generate an improved solution that addresses these issues. \
             Provide your response as a JSON ActionRequest object.",
            eval_report.task_id,
            eval_report.score,
            if eval_report.status.as_ref().map(|s| s.to_string()).unwrap_or_default() == "pass" { "PASSING" } else { "FAILING" },
            eval_report.logs.join("\n"),
            improvement_suggestions
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{TaskType};

    #[test]
    fn test_generate_action_request_valid() {
        let strategy = AdaptivePromptingStrategy::new();
        let task = crate::types::Task::new("test task".to_string(), TaskType::CodeGeneration);

        let valid_json = r#"{
            "action_type": "write",
            "changeset": {
                "patches": [{
                    "path": "test.rs",
                    "hunks": [{
                        "old_start": 1,
                        "old_lines": 0,
                        "new_start": 1,
                        "new_lines": 1,
                        "lines": "+fn main() {}\n"
                    }],
                    "expected_prev_sha256": null
                }]
            },
            "reason": "Generated main function",
            "confidence": 0.95,
            "metadata": {}
        }"#;

        // This would normally be an async test, but we're testing the parsing logic
        // In a real test, we'd call generate_action_request
        let action_request: ActionRequest = serde_json::from_str(valid_json).unwrap();
        assert!(action_request.validate().is_ok());
        assert_eq!(action_request.action_type, crate::types::ActionType::Write);
        assert_eq!(action_request.confidence, 0.95);
    }

    #[test]
    fn test_generate_initial_prompt() {
        let strategy = AdaptivePromptingStrategy::new();
        let task = crate::types::Task::new("Implement user authentication".to_string(), TaskType::CodeGeneration);

        let prompt = strategy.generate_initial_prompt(&task);

        assert!(prompt.contains("Implement user authentication"));
        assert!(prompt.contains("JSON ActionRequest"));
        assert!(prompt.contains("action_type"));
        assert!(prompt.contains("changeset"));
    }

    #[test]
    fn test_generate_refinement_prompt() {
        let strategy = AdaptivePromptingStrategy::new();

        // Create a mock eval report
        let eval_report = EvalReport {
            task_id: "test-task".to_string(),
            artifact_paths: vec![],
            status: crate::evaluation::EvalStatus::Fail,
            score: 0.3,
            thresholds_met: vec![],
            thresholds_missed: vec!["tests-pass".to_string()],
            criteria: vec![
                crate::evaluation::EvalCriterion {
                    id: "tests-pass".to_string(),
                    description: "Tests should pass".to_string(),
                    weight: 1.0,
                    passed: false,
                    score: 0.0,
                    notes: Some("Test failure: assertion error".to_string()),
                }
            ],
            iterations: 1,
            prompt_tokens: None,
            completion_tokens: None,
            elapsed_ms: Some(1000),
            stop_reason: None,
            next_actions: vec![],
            logs: vec!["Test execution failed".to_string()],
            seed: None,
            tool_versions: std::collections::HashMap::new(),
            timestamp: chrono::Utc::now(),
        };

        let prompt = strategy.generate_refinement_prompt(&eval_report);

        assert!(prompt.contains("test-task"));
        assert!(prompt.contains("0.30"));
        assert!(prompt.contains("FAILING"));
        assert!(prompt.contains("JSON ActionRequest"));
    }
}