//! Documentation Quality Validator Usage Example
//!
//! Demonstrates how to use the documentation quality validator
//! in the V3 Rust architecture.

use agent_agency_mcp::{
    tools::DocQualityValidator,
    types::*,
    ToolRegistry,
};
use anyhow::Result;
use serde_json::json;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🔍 Documentation Quality Validator Example");
    println!("==========================================");

    // Create tool registry with documentation quality validator
    let tool_registry = ToolRegistry::new();
    tool_registry.initialize().await?;

    // Create a test request for documentation quality validation
    let test_content = r#"# My Amazing Project

This is a **revolutionary breakthrough** in AI technology! Our system is **production-ready** and provides **enterprise-grade** solutions.

## Features

- ✅ **100% complete** implementation
- ✅ **Cutting-edge** technology
- ✅ **Industry-leading** performance
- ✅ **Award-winning** design

## Status

**Production Ready** - All features implemented and tested.

## Next Steps

- Deploy to production
- Scale to millions of users
- Achieve world domination
"#;

    let request = ToolExecutionRequest {
        id: Uuid::new_v4(),
        tool_id: Uuid::new_v4(), // Will be set by the registry
        parameters: json!({
            "content": test_content,
            "content_type": "markdown",
            "validation_level": "strict",
            "include_suggestions": true
        }),
        timeout_seconds: Some(30),
        priority: ExecutionPriority::Normal,
        metadata: std::collections::HashMap::new(),
    };

    println!("\n📝 Testing Documentation Quality Validation");
    println!("----------------------------------------");

    // Execute the documentation quality validator
    let result = tool_registry.execute_tool(request).await?;

    println!("✅ Validation completed!");
    println!("Status: {:?}", result.status);
    println!("Duration: {:?}ms", result.duration_ms);

    if let Some(output) = result.output {
        let quality_result: DocQualityResult = serde_json::from_value(output)?;
        
        println!("\n📊 Quality Results");
        println!("==================");
        println!("Quality Score: {:.2}", quality_result.quality_score);
        println!("Issues Found: {}", quality_result.issues.len());
        
        println!("\n📈 Metrics");
        println!("----------");
        println!("Superiority Claims: {}", quality_result.metrics.superiority_claims);
        println!("Unfounded Achievements: {}", quality_result.metrics.unfounded_achievements);
        println!("Marketing Language: {}", quality_result.metrics.marketing_language);
        println!("Temporal Docs: {}", quality_result.metrics.temporal_docs);
        println!("Emoji Usage: {}", quality_result.metrics.emoji_usage);
        
        if !quality_result.issues.is_empty() {
            println!("\n🚨 Issues Found");
            println!("---------------");
            for (i, issue) in quality_result.issues.iter().enumerate() {
                println!("{}. [{}] {}: {}", 
                    i + 1, 
                    format!("{:?}", issue.severity).to_lowercase(),
                    issue.rule_id,
                    issue.message
                );
                if !issue.suggested_fix.is_empty() {
                    println!("   💡 Suggestion: {}", issue.suggested_fix);
                }
            }
        }
        
        if !quality_result.recommendations.is_empty() {
            println!("\n💡 Recommendations");
            println!("------------------");
            for (i, rec) in quality_result.recommendations.iter().enumerate() {
                println!("{}. {}", i + 1, rec);
            }
        }
    }

    // Test with better content
    println!("\n\n🔧 Testing with Improved Content");
    println!("================================");

    let improved_content = r#"# My Project

This project implements an AI orchestration system with constitutional governance, multiple execution modes, and monitoring capabilities.

## Features

- User authentication with JWT tokens
- Real-time notifications via WebSocket
- Basic analytics dashboard
- Task execution pipeline
- MCP tool ecosystem

## Status

**Implemented** - Core functionality working with monitoring capabilities.

## Architecture

The system uses a modular architecture with:
- Rust-based core for performance
- Constitutional council for governance
- MCP integration for tool access
- PostgreSQL for persistence
"#;

    let improved_request = ToolExecutionRequest {
        id: Uuid::new_v4(),
        tool_id: Uuid::new_v4(),
        parameters: json!({
            "content": improved_content,
            "content_type": "markdown",
            "validation_level": "strict",
            "include_suggestions": true
        }),
        timeout_seconds: Some(30),
        priority: ExecutionPriority::Normal,
        metadata: std::collections::HashMap::new(),
    };

    let improved_result = tool_registry.execute_tool(improved_request).await?;

    if let Some(output) = improved_result.output {
        let quality_result: DocQualityResult = serde_json::from_value(output)?;
        
        println!("✅ Improved Content Validation");
        println!("Quality Score: {:.2}", quality_result.quality_score);
        println!("Issues Found: {}", quality_result.issues.len());
        
        if quality_result.quality_score > 0.8 {
            println!("🎉 Excellent! This content meets quality standards.");
        } else if quality_result.quality_score > 0.6 {
            println!("👍 Good! Minor improvements needed.");
        } else {
            println!("⚠️ Needs improvement to meet quality standards.");
        }
    }

    // Demonstrate agent workflow integration
    println!("\n\n🤖 Agent Workflow Integration Example");
    println!("=====================================");

    let agent_workflow = AgentWorkflowExample::new(tool_registry);
    agent_workflow.demonstrate_autonomous_documentation_creation().await?;

    Ok(())
}

/// Example of how agents can use the documentation quality validator
struct AgentWorkflowExample {
    tool_registry: ToolRegistry,
}

impl AgentWorkflowExample {
    fn new(tool_registry: ToolRegistry) -> Self {
        Self { tool_registry }
    }

    async fn demonstrate_autonomous_documentation_creation(&self) -> Result<()> {
        println!("🤖 Agent: Creating documentation autonomously...");
        
        // Simulate agent creating documentation
        let mut documentation = r#"# My New Feature

This is a **revolutionary** new feature that will **change everything**!

## Status: **Production Ready**

We have **100% complete** implementation with **enterprise-grade** quality.
"#.to_string();

        let mut iterations = 0;
        let max_iterations = 3;

        while iterations < max_iterations {
            println!("🔄 Agent: Validating documentation (iteration {})", iterations + 1);
            
            let request = ToolExecutionRequest {
                id: Uuid::new_v4(),
                tool_id: Uuid::new_v4(),
                parameters: json!({
                    "content": documentation,
                    "content_type": "markdown",
                    "validation_level": "moderate",
                    "include_suggestions": true
                }),
                timeout_seconds: Some(30),
                priority: ExecutionPriority::Normal,
                metadata: std::collections::HashMap::new(),
            };

            let result = self.tool_registry.execute_tool(request).await?;

            if let Some(output) = result.output {
                let quality_result: DocQualityResult = serde_json::from_value(output)?;
                
                println!("📊 Quality Score: {:.2}", quality_result.quality_score);
                
                if quality_result.quality_score >= 0.8 {
                    println!("✅ Agent: Documentation quality is acceptable!");
                    break;
                } else {
                    println!("🔧 Agent: Improving documentation based on feedback...");
                    
                    // Simulate agent applying suggestions
                    documentation = self.apply_quality_suggestions(&documentation, &quality_result.issues);
                }
            }
            
            iterations += 1;
        }

        if iterations >= max_iterations {
            println!("⚠️ Agent: Reached maximum iterations, proceeding with current quality");
        }

        Ok(())
    }

    fn apply_quality_suggestions(&self, content: &str, issues: &[QualityIssue]) -> String {
        let mut improved = content.to_string();
        
        // Apply common fixes
        improved = improved.replace("revolutionary", "innovative");
        improved = improved.replace("Production Ready", "Implemented");
        improved = improved.replace("100% complete", "implemented");
        improved = improved.replace("enterprise-grade", "robust");
        
        println!("🔧 Applied quality improvements based on {} issues", issues.len());
        
        improved
    }
}

/// Documentation quality result structure for deserialization
#[derive(Debug, serde::Deserialize)]
struct DocQualityResult {
    validation_id: String,
    quality_score: f64,
    issues: Vec<QualityIssue>,
    metrics: QualityMetrics,
    recommendations: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct QualityIssue {
    severity: String,
    rule_id: String,
    message: String,
    line_number: u32,
    suggested_fix: String,
}

#[derive(Debug, serde::Deserialize)]
struct QualityMetrics {
    superiority_claims: u32,
    unfounded_achievements: u32,
    marketing_language: u32,
    temporal_docs: u32,
    emoji_usage: u32,
}
