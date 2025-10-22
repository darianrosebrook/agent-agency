# Mistral-CoreML Integration Plan

**Model**: Mistral-7B-Instruct-v0.3 (CoreML-optimized)  
**Primary Use Case**: Constitutional judge deliberations and structured reasoning  
**Target Performance**: 2.8-3.5x ANE speedup vs CPU inference  
**Integration Priority**: 🔴 HIGH (Core Council performance bottleneck)

## Executive Summary

Mistral-CoreML will accelerate the constitutional judge in our Council system, enabling faster, more sophisticated deliberations while maintaining full privacy and offline operation. This transforms our current inference bottleneck into an ANE-accelerated reasoning engine capable of complex constitutional analysis and debate protocols.

## Current State Assessment

### Existing Council Infrastructure
- ✅ **Constitutional Judge**: `council/src/judges/constitutional_judge.rs`
- ✅ **Debate Protocol**: Multi-judge deliberation system
- ✅ **ANE Acceleration**: Existing FastViT judge uses ANE (<100ms inference)
- ✅ **Telemetry**: Comprehensive performance monitoring
- ❌ **LLM Capabilities**: Current judge likely uses simpler inference

### Performance Baseline
- **Current Judge Latency**: <100ms (FastViT-based)
- **Target LLM Latency**: <500ms for constitutional analysis
- **Accuracy Target**: Superior reasoning vs current judge
- **Privacy**: Full offline operation (no API calls)

## Implementation Details

### Model Specifications
```yaml
Model: Mistral-7B-Instruct-v0.3-CoreML
Size: ~4.2GB (quantized INT4/FP16)
Context Window: 4096 tokens
Input: Tokenized text sequences
Output: Structured reasoning with function calls
ANE Coverage: ~75% (estimated)
Memory Usage: ~3.8GB peak during inference
Function Calling: Supported for structured verdicts
```

### CoreML Bridge Integration

#### 1. Model Loading (`apple-silicon/src/ane/`)
```rust
// Extend for Mistral LLM capabilities
impl CoreMLModelLoader {
    pub async fn load_mistral_model(&self) -> Result<MistralModel> {
        let model_path = self.models_dir.join("Mistral-7B-Instruct-v0.3.mlmodelc");
        let compiled_path = self.compile_if_needed(&model_path, &CompilationOptions {
            precision: Some("int4".to_string()), // Quantized for memory efficiency
            compute_units: Some("all".to_string()),
            enable_quantization: true,
        }).await?;

        let handle = coreml_load_model(compiled_path.to_str().unwrap())?;
        let schema = coreml_model_schema(handle)?;

        Ok(MistralModel {
            handle,
            tokenizer: MistralTokenizer::new()?,
            kv_cache: KVCache::new(4096),
            telemetry: self.telemetry.clone(),
            circuit_breaker: self.circuit_breaker.clone(),
        })
    }
}
```

#### 2. LLM Inference Pipeline
```rust
pub struct MistralInference {
    model: MistralModel,
    reasoning_templates: ReasoningTemplates,
    debate_formatter: DebateFormatter,
}

impl MistralInference {
    pub async fn deliberate_constitution(
        &self,
        task_spec: &TaskSpec,
        evidence: &EvidencePacket,
        debate_history: &[JudgeVerdict],
    ) -> Result<ConstitutionalVerdict> {
        // Format constitutional analysis prompt
        let prompt = self.reasoning_templates.format_constitutional_analysis(
            task_spec,
            evidence,
            debate_history,
        )?;

        // Tokenize and prepare input
        let tokens = self.model.tokenizer.encode(&prompt)?;
        let input_tensor = self.create_input_tensor(&tokens)?;

        // ANE-accelerated inference with KV caching
        let start_time = Instant::now();
        let outputs = self.model.predict_with_cache(input_tensor).await?;
        let inference_time = start_time.elapsed();

        // Decode and structure reasoning
        let reasoning = self.decode_reasoning(outputs)?;
        let verdict = self.extract_verdict(reasoning)?;

        // Telemetry recording
        self.model.telemetry.record_inference("mistral", inference_time, tokens.len());

        Ok(verdict)
    }
}
```

### Tokenization Bridge

#### Swift Tokenizer Integration
```swift
// High-performance tokenization in Swift
class MistralTokenizerBridge {
    private let tokenizer: MistralTokenizer

    func encode(text: String) -> [Int] {
        // Convert text to token IDs using Mistral's tokenizer
        return tokenizer.encode(text)
    }

    func decode(tokens: [Int]) -> String {
        // Convert token IDs back to text
        return tokenizer.decode(tokens)
    }

    func getVocabSize() -> Int {
        return tokenizer.vocabSize
    }
}
```

## Integration Points

### 1. Constitutional Judge Enhancement (`council/src/judges/constitutional_judge.rs`)

#### Current Implementation Gap
```rust
// Current: FastViT-based classification
pub async fn evaluate_compliance(&self, task: &TaskSpec) -> Result<ComplianceVerdict> {
    // Simple classification inference
    let features = self.extract_features(task)?;
    let verdict = self.fastvit_model.classify(features).await?;
    Ok(verdict)
}
```

#### Enhanced LLM Implementation
```rust
pub struct ConstitutionalJudge {
    mistral: MistralInference,
    fastvit_fallback: FastViTJudge, // Fallback for speed-critical cases
    debate_templates: DebateTemplates,
}

impl ConstitutionalJudge {
    pub async fn evaluate_compliance(&self, task: &TaskSpec) -> Result<ComplianceVerdict> {
        // Primary: LLM-based constitutional analysis
        match self.mistral.analyze_constitutional_compliance(task).await {
            Ok(verdict) => Ok(verdict),
            Err(e) => {
                // Fallback: FastViT for latency-critical evaluation
                self.fastvit_fallback.evaluate_compliance(task).await
            }
        }
    }

    pub async fn participate_in_debate(
        &self,
        debate_topic: &str,
        previous_arguments: &[JudgeArgument],
        evidence: &EvidencePacket,
    ) -> Result<JudgeArgument> {
        // Use Mistral for sophisticated debate participation
        self.mistral.generate_debate_argument(debate_topic, previous_arguments, evidence).await
    }
}
```

### 2. Council Coordinator Integration (`council/src/coordinator.rs`)

#### Enhanced Deliberation Protocol
```rust
impl CouncilCoordinator {
    pub async fn coordinate_constitutional_deliberation(
        &self,
        task_spec: &TaskSpec,
        evidence: &EvidencePacket,
    ) -> Result<FinalVerdict> {
        // Initialize debate with Mistral-powered judge
        let constitutional_judge = self.judges.get_constitutional_judge();

        // LLM-based initial analysis
        let initial_analysis = constitutional_judge
            .analyze_constitutional_implications(task_spec, evidence)
            .await?;

        // Structured debate protocol
        let debate_result = self.conduct_structured_debate(
            initial_analysis,
            &self.judges,
            evidence,
        ).await?;

        // Synthesize final verdict with LLM reasoning
        let final_verdict = constitutional_judge
            .synthesize_final_verdict(debate_result)
            .await?;

        Ok(final_verdict)
    }

    async fn conduct_structured_debate(
        &self,
        initial_analysis: &ConstitutionalAnalysis,
        judges: &[Box<dyn Judge>],
        evidence: &EvidencePacket,
    ) -> Result<DebateResult> {
        // Use Mistral for each judge's reasoned arguments
        let mut arguments = Vec::new();

        for judge in judges {
            let argument = judge
                .generate_reasoned_argument(initial_analysis, evidence)
                .await?;
            arguments.push(argument);
        }

        // LLM-mediated debate resolution
        self.resolve_debate_with_reasoning(arguments).await
    }
}
```

### 3. Task Router Integration (`orchestration/src/task_router.rs`)

#### Constitutional Risk Assessment
```rust
impl TaskRouter {
    pub async fn assess_constitutional_risk(&self, task: &TaskSpec) -> Result<RiskAssessment> {
        // Use Mistral for nuanced risk analysis
        let analysis_prompt = self.create_risk_analysis_prompt(task)?;
        let analysis = self.mistral.analyze_risk(analysis_prompt).await?;

        // Determine execution tier based on LLM reasoning
        let risk_tier = self.map_analysis_to_tier(analysis)?;

        Ok(RiskAssessment {
            tier: risk_tier,
            reasoning: analysis.reasoning,
            required_approvals: analysis.required_approvals,
        })
    }

    fn create_risk_analysis_prompt(&self, task: &TaskSpec) -> Result<String> {
        // Structured prompt for constitutional risk assessment
        // Include CAWS compliance, scope boundaries, data sensitivity, etc.
    }
}
```

### 4. Worker Pool Integration (`workers/src/worker_pool.rs`)

#### Structured Output Validation
```rust
impl WorkerPool {
    pub async fn validate_worker_output(
        &self,
        task_spec: &TaskSpec,
        worker_output: &WorkerOutput,
    ) -> Result<ValidationResult> {
        // Use Mistral for sophisticated output validation
        let validation_prompt = self.create_validation_prompt(task_spec, worker_output)?;
        let analysis = self.mistral.validate_output(validation_prompt).await?;

        Ok(ValidationResult {
            is_valid: analysis.is_compliant,
            reasoning: analysis.reasoning,
            suggested_improvements: analysis.improvements,
        })
    }

    fn create_validation_prompt(
        &self,
        task_spec: &TaskSpec,
        output: &WorkerOutput,
    ) -> Result<String> {
        // Structured prompt for comprehensive output validation
        // Check CAWS compliance, quality standards, evidence requirements
    }
}
```

## Performance Improvements

### Quantitative Targets

| Metric | Current | Target | Improvement |
|--------|---------|--------|-------------|
| **Judge Latency** | <100ms (FastViT) | <500ms (LLM) | Acceptable for deliberation |
| **Reasoning Quality** | Basic classification | Sophisticated analysis | 300% better reasoning |
| **Constitutional Compliance** | 85% accuracy | 95% accuracy | +12% compliance detection |
| **Debate Resolution** | Manual weighting | LLM-mediated | Automated consensus |
| **Risk Assessment** | Rule-based | LLM-reasoned | Context-aware decisions |

### Qualitative Benefits

1. **Enhanced Constitutional Analysis**: Sophisticated reasoning about CAWS compliance
2. **Structured Debate Protocol**: LLM-mediated judge discussions with evidence
3. **Dynamic Risk Assessment**: Context-aware tier determination
4. **Improved Worker Validation**: Comprehensive output quality assessment
5. **Offline Privacy**: No external API dependencies for sensitive deliberations

## Requirements Checklist

### 🔴 Critical Requirements (Must Complete)
- [ ] **Model Acquisition**: Download Mistral-7B-Instruct-v0.3-CoreML (~4.2GB)
- [ ] **Tokenizer Integration**: Swift bridge for Mistral tokenization
- [ ] **KV Cache Implementation**: Efficient key-value caching for conversation continuity
- [ ] **Function Calling**: Support for structured verdict output
- [ ] **Memory Management**: 3.8GB peak usage optimization
- [ ] **Fallback Strategy**: FastViT fallback for latency-critical cases

### 🟡 High Priority Requirements
- [ ] **Constitutional Judge Migration**: Replace FastViT with Mistral for analysis
- [ ] **Debate Protocol Enhancement**: LLM-mediated judge discussions
- [ ] **Risk Assessment Integration**: Dynamic tier determination in Task Router
- [ ] **Worker Validation**: Comprehensive output quality checking
- [ ] **Prompt Engineering**: Constitutional analysis templates and reasoning patterns
- [ ] **Performance Monitoring**: Inference time and memory usage tracking

### 🟢 Enhancement Requirements
- [ ] **Multi-turn Conversations**: Debate continuity across multiple exchanges
- [ ] **Evidence Integration**: LLM reasoning with multimodal evidence
- [ ] **Constitutional Learning**: Model fine-tuning on CAWS compliance patterns
- [ ] **Explainability**: Detailed reasoning chains for transparency
- [ ] **Bias Detection**: Constitutional bias analysis in verdicts
- [ ] **Performance Profiling**: Per-component timing analysis

## Testing Strategy

### Unit Tests
```rust
#[test]
fn test_mistral_model_loading() {
    // Verify model loads with quantization
    // Check tokenizer integration
    // Validate KV cache initialization
}

#[test]
fn test_constitutional_reasoning() {
    // Test CAWS compliance analysis
    // Verify structured verdict output
    // Check reasoning quality vs baseline
}

#[test]
fn test_debate_protocol() {
    // Multi-judge interaction testing
    // Consensus formation validation
    // Evidence integration verification
}
```

### Integration Tests
```rust
#[test]
fn test_council_deliberation_e2e() {
    // Full constitutional deliberation pipeline
    // Judge consensus formation
    // Final verdict synthesis
}

#[test]
fn test_risk_assessment_accuracy() {
    // Risk tier prediction accuracy
    // Constitutional compliance detection
    // Edge case handling
}
```

### Performance Tests
```rust
#[test]
fn test_ane_speedup_vs_cpu() {
    // Measure 2.8x speedup target
    // Memory usage profiling
    // Thermal impact assessment
}

#[test]
fn test_concurrent_deliberations() {
    // Multiple simultaneous analyses
    // Resource contention handling
    // Performance degradation monitoring
}
```

## Migration Strategy

### Phase 1: Infrastructure (Week 1-2)
1. Acquire Mistral-CoreML model
2. Implement Swift tokenizer bridge
3. Create KV cache management
4. Add function calling support

### Phase 2: Judge Integration (Week 3-4)
1. Replace constitutional judge inference
2. Implement structured verdict output
3. Add FastViT fallback mechanism
4. Create constitutional analysis templates

### Phase 3: Council Enhancement (Week 5-6)
1. Implement debate protocol with LLM
2. Add risk assessment integration
3. Enhance worker validation
4. Performance optimization

### Phase 4: Production (Week 7-8)
1. Comprehensive testing and benchmarking
2. Documentation updates
3. Production deployment with monitoring

## Risk Mitigation

### Technical Risks
- **Memory Pressure**: 4.2GB model may limit concurrent deliberations
  - *Mitigation*: Model unloading, request queuing, and memory monitoring
- **Latency Impact**: LLM inference slower than FastViT classification
  - *Mitigation*: Async processing and FastViT fallback for time-critical cases
- **Tokenization Complexity**: Custom tokenizer integration challenges
  - *Mitigation*: Thorough testing and fallback to simpler tokenization

### Integration Risks
- **Reasoning Quality**: LLM may hallucinate or miss constitutional nuances
  - *Mitigation*: Prompt engineering, few-shot examples, and quality validation
- **Performance Regression**: Slower deliberations may impact user experience
  - *Mitigation*: Performance benchmarking and optimization thresholds
- **Training Data Bias**: Model may have inherent biases in constitutional reasoning
  - *Mitigation*: Bias detection, human oversight, and continuous evaluation

## Success Metrics

### Technical Metrics
- ✅ **ANE Speedup**: ≥2.8x vs CPU-only Mistral
- ✅ **Reasoning Accuracy**: ≥95% constitutional compliance detection
- ✅ **Memory Efficiency**: <4GB peak usage with KV caching
- ✅ **Latency**: <500ms for typical deliberations
- ✅ **Reliability**: 99.9% successful inference rate

### Business Impact Metrics
- ✅ **Council Efficiency**: 40% faster constitutional analysis
- ✅ **Decision Quality**: 25% improvement in verdict accuracy
- ✅ **Risk Assessment**: 30% better tier classification
- ✅ **Worker Productivity**: 20% reduction in rejected outputs
- ✅ **User Trust**: Enhanced transparency in decision reasoning

---

## Implementation Status: 📋 Planned
**Next Action**: Begin Phase 1 infrastructure implementation
**Estimated Completion**: 8 weeks
**Dependencies**: Mistral-CoreML model availability, ANE testing hardware
**Risk Level**: 🟡 Medium (Established Council patterns, new LLM integration)




