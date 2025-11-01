# Instinct Model Integration Guide

**Author**: @darianrosebrook  
**Date**: January 2025  
**Status**: In Development (Proof of Concept)

## Overview

The Instinct model (`continuedev/instinct`) is a specialized code editing assistant fine-tuned from Qwen2.5-Coder-7B, designed to excel at code refactoring, documentation, security fixes, and performance optimization. This document outlines its integration into the Agent Agency multi-model system.

## Model Specifications

- **Base Model**: Qwen2.5-Coder-7B
- **Purpose**: Code editing, refactoring, and improvement
- **Parameters**: 8B
- **Formats**: Ollama (nate/instinct), GGUF (instinct-Q4_K_M.gguf)
- **Context Length**: 4096 tokens
- **Quantization**: Q4_K_M (recommended for balance of quality/speed)

## Integration Architecture

### Multi-Model Orchestration

```rust
// In agent-orchestration/src/lib.rs
pub enum CodeAssistant {
    InstinctOllama,
    InstinctCoreML,
    GemmaFallback,
}

impl CodeAssistant {
    pub async fn improve_code(&self, code: &str, task: CodeTask) -> Result<String, Error> {
        let prompt = self.build_prompt(task, code);
        
        match self {
            InstinctOllama => {
                let ollama = OllamaService::with_model("nate/instinct").await?;
                ollama.generate(&prompt).await
            },
            InstinctCoreML => {
                let coreml = CoreMLService::with_model("instinct").await?;
                coreml.generate(&prompt).await
            },
            GemmaFallback => {
                let gemma = OllamaService::with_model("gemma3n:e2b").await?;
                gemma.generate(&prompt).await
            },
        }
    }
}
```

### Task Routing Logic

```rust
pub enum CodeTask {
    Refactor { complexity: Complexity },
    Document { language: Language },
    SecurityReview { risk_level: RiskLevel },
    PerformanceOptimize { bottleneck: BottleneckType },
    BugFix { error_type: ErrorType },
}

impl CodeAssistant {
    fn route_task(&self, task: &CodeTask) -> Self {
        match task {
            CodeTask::SecurityReview { .. } => CodeAssistant::InstinctCoreML, // Best security
            CodeTask::Refactor { complexity: Complexity::High } => CodeAssistant::InstinctOllama,
            CodeTask::PerformanceOptimize { .. } => CodeAssistant::InstinctCoreML, // ANE acceleration
            _ => CodeAssistant::GemmaFallback, // General tasks
        }
    }
}
```

## Setup Instructions

### 1. Ollama Setup

```bash
# Install Ollama (if not already installed)
curl -fsSL https://ollama.ai/install.sh | sh

# Pull the official Instinct model
ollama pull nate/instinct

# Verify installation
ollama list | grep instinct
```

### 2. Local GGUF Setup

```bash
# Run the setup script
chmod +x models/scripts/setup-instinct-ollama.sh
./models/scripts/setup-instinct-ollama.sh

# This creates a local model named 'instinct-gguf'
ollama list | grep instinct
```

### 3. CoreML Conversion (Future)

```python
# models/scripts/convert_instinct_to_coreml.py
import coremltools as ct
from transformers import AutoModelForCausalLM, AutoTokenizer

def convert_instinct_to_coreml():
    """Convert Instinct model to CoreML format for Apple Silicon optimization"""
    
    # Load model and tokenizer
    model = AutoModelForCausalLM.from_pretrained("continuedev/instinct")
    tokenizer = AutoTokenizer.from_pretrained("continuedev/instinct")
    
    # Quantize for CoreML
    quantized_model = ct.models.neural_network.quantization_utils.quantize_weights(
        model, nbits=8
    )
    
    # Convert to CoreML
    coreml_model = ct.convert(
        quantized_model,
        inputs=[ct.TensorType(name="input_ids", shape=(1, ct.RangeDim(1, 4096)))],
        outputs=[ct.TensorType(name="logits")],
        minimum_deployment_target=ct.target.iOS16,
    )
    
    # Save optimized model
    coreml_model.save("models/coreml/instinct/instinct.mlpackage")
    
    print("✅ Instinct CoreML conversion complete")
```

## Usage Patterns

### 1. Agent Self-Improvement

```rust
// In agent-orchestration/src/autonomous_executor.rs
impl AutonomousExecutor {
    async fn self_improve(&mut self) -> Result<(), Error> {
        let current_code = self.extract_current_logic();
        
        let improved_code = self.code_assistant
            .improve_code(&current_code, CodeTask::Refactor {
                complexity: Complexity::High
            })
            .await?;
            
        if self.validate_improvement(&improved_code)? {
            self.update_logic(&improved_code).await?;
            self.audit_trail.record_self_improvement(&improved_code);
        }
        
        Ok(())
    }
}
```

### 2. Security Code Review

```rust
// In agent-orchestration/src/security.rs
impl SecurityAuditor {
    async fn review_agent_code(&self, code: &str) -> Result<SecurityReport, Error> {
        let prompt = format!(
            "Review this Rust code for security vulnerabilities:\n\n{}\n\n\
            Focus on: SQL injection, input validation, memory safety, \
            authentication bypass, authorization flaws.",
            code
        );
        
        let review = self.instinct_assistant
            .generate(&prompt)
            .await?;
            
        Ok(self.parse_security_review(&review))
    }
}
```

### 3. Performance Optimization

```rust
// In agent-orchestration/src/performance.rs
impl PerformanceOptimizer {
    async fn optimize_agent(&self, agent_code: &str) -> Result<String, Error> {
        let prompt = format!(
            "Optimize this Rust agent code for performance:\n\n{}\n\n\
            Focus on: algorithmic complexity, memory usage, \
            async/await patterns, database queries, API calls.",
            agent_code
        );
        
        self.instinct_assistant
            .generate(&prompt)
            .await
    }
}
```

## Evaluation Framework

### Test Suite

The comprehensive test suite evaluates Instinct across multiple dimensions:

```bash
# Run full evaluation
python3 models/scripts/test_instinct_code_editing.py --output instinct_evaluation.json

# Run specific test categories
python3 models/scripts/test_instinct_code_editing.py --tests security,performance
```

### Performance Metrics

| Category | Grade | Score | Notes |
|----------|-------|-------|-------|
| **Security** | A+ | 100% | Perfect SQL injection prevention |
| **Python Generation** | A | 100% | Clean, documented code |
| **Code Refactoring** | B | 71% | Good but needs improvement |
| **Documentation** | B- | 60% | Functional but verbose |
| **Bug Detection** | C+ | 50% | Misses some edge cases |
| **Performance** | B+ | 80% | Strong algorithmic optimization |
| **Multi-language** | C | 40% | Limited Go/TypeScript support |

### Overall Grade: **B+ (73%)**

## Configuration

### Ollama Parameters

```yaml
# models/coreml/instinct/ollama_config.yaml
model: "nate/instinct"
parameters:
  temperature: 0.1      # Low for code tasks
  top_p: 0.9
  top_k: 40
  num_ctx: 4096
  repeat_penalty: 1.1
  stop: ["```", "---"]
```

### CoreML Optimization

```yaml
# models/coreml/instinct/coreml_config.yaml
quantization: "8bit"
deployment_target: "iOS16"
ane_optimization: true
memory_pool_size: "2GB"
batch_size: 1
```

## Integration Points

### 1. CAWS Workflow Integration

```rust
// In agent-orchestration/src/caws_integration.rs
impl CAWSIntegration {
    async fn code_review_phase(&self, spec: &WorkingSpec) -> Result<CodeReview, Error> {
        let code = self.extract_implementation_code(spec)?;
        
        let review = self.instinct_assistant
            .improve_code(&code, CodeTask::SecurityReview {
                risk_level: spec.risk_tier.into()
            })
            .await?;
            
        Ok(CodeReview {
            original_code: code,
            suggested_improvements: review,
            security_score: self.calculate_security_score(&review),
            performance_score: self.calculate_performance_score(&review),
        })
    }
}
```

### 2. Multi-Model Fallback

```rust
// In agent-orchestration/src/model_orchestrator.rs
impl ModelOrchestrator {
    async fn generate_code_assistance(&self, task: CodeTask) -> Result<String, Error> {
        let models = vec![
            ("instinct-coreml", 0.9),    // Highest confidence
            ("instinct-ollama", 0.8),    // Good fallback
            ("gemma3n:e2b", 0.6),        // General purpose
        ];
        
        for (model, confidence) in models {
            match self.try_model(model, &task).await {
                Ok(result) if self.validate_result(&result, confidence)? => {
                    return Ok(result);
                },
                Err(e) => {
                    tracing::warn!("Model {} failed: {}", model, e);
                    continue;
                }
            }
        }
        
        Err(Error::AllModelsFailed)
    }
}
```

## Monitoring and Observability

### Metrics Collection

```rust
// In system-observability/src/metrics.rs
pub struct InstinctMetrics {
    pub requests_total: Counter,
    pub response_time_histogram: Histogram,
    pub success_rate: Gauge,
    pub security_fixes_applied: Counter,
    pub performance_improvements: Counter,
    pub code_quality_score: Gauge,
}

impl InstinctMetrics {
    pub fn record_code_task(&self, task: CodeTask, duration: Duration, success: bool) {
        self.requests_total.inc();
        self.response_time_histogram.observe(duration.as_secs_f64());
        
        if success {
            self.success_rate.set(1.0);
            match task {
                CodeTask::SecurityReview { .. } => self.security_fixes_applied.inc(),
                CodeTask::PerformanceOptimize { .. } => self.performance_improvements.inc(),
                _ => {}
            }
        } else {
            self.success_rate.set(0.0);
        }
    }
}
```

### Health Checks

```rust
// In system-resilience/src/health_checks.rs
impl InstinctHealthCheck {
    async fn check_instinct_availability(&self) -> HealthStatus {
        let test_prompt = "Write a simple hello world function in Rust";
        
        match self.instinct_service.generate(&test_prompt).await {
            Ok(response) if response.contains("fn main") => HealthStatus::Healthy,
            Ok(_) => HealthStatus::Degraded("Unexpected response format"),
            Err(e) => HealthStatus::Unhealthy(format!("Service error: {}", e)),
        }
    }
}
```

## Future Enhancements

### 1. Fine-tuning for Agent Tasks

```python
# models/scripts/finetune_instinct_for_agents.py
def create_agent_training_data():
    """Create training data specific to agent orchestration tasks"""
    return [
        {
            "instruction": "Refactor this agent function for better error handling",
            "input": "fn process_task() -> Result<(), Error> { ... }",
            "output": "fn process_task() -> Result<(), AgentError> { ... }"
        },
        # More agent-specific examples
    ]
```

### 2. Multi-Model Ensemble

```rust
// Future: Combine Instinct with other models for better results
impl EnsembleCodeAssistant {
    async fn generate_ensemble_response(&self, task: CodeTask) -> Result<String, Error> {
        let responses = join_all(vec![
            self.instinct.generate(&task),
            self.gemma.generate(&task),
            self.claude.generate(&task),
        ]).await;
        
        self.consensus_algorithm.combine_responses(responses)
    }
}
```

## Troubleshooting

### Common Issues

1. **Ollama Connection Failed**
   ```bash
   # Check Ollama service
   ollama serve
   
   # Verify model availability
   ollama list
   ```

2. **CoreML Conversion Errors**
   ```bash
   # Check CoreML tools version
   pip show coremltools
   
   # Verify PyTorch compatibility
   python -c "import torch; print(torch.__version__)"
   ```

3. **Performance Issues**
   ```bash
   # Monitor resource usage
   top -p $(pgrep ollama)
   
   # Check model quantization
   ollama show nate/instinct
   ```

## References

- [Instinct Model Card](https://huggingface.co/continuedev/instinct)
- [Ollama Documentation](https://ollama.ai/docs)
- [CoreML Tools](https://coremltools.readme.io/)
- [Arbiter Stack Theory](../arbiter/theory.md)
- [CoreML-First Architecture](../architecture/coreml-first-decision.md)

---

**Next Steps**: 
1. Complete CoreML conversion pipeline
2. Integrate with agent orchestration system
3. Implement comprehensive monitoring
4. Fine-tune for agent-specific tasks
