# CoreML Integration for Constitutional Council Judges

## Overview

All four constitutional council judges are fully integrated with CoreML inference through the `JudgeEngine` trait. The judges are engine-agnostic and work with any implementation of `JudgeEngine`, including `CoreMLEngine`.

## Judge Integration Status

All four judges use CoreML inference:

- **ConstitutionalJudge** - Uses `JudgeEngine` for ethical and compliance analysis
- **TechnicalAuditor** - Uses `JudgeEngine` for code quality and security review  
- **QualityEvaluator** - Uses `JudgeEngine` for testing and requirements completeness
- **IntegrationValidator** - Uses `JudgeEngine` for API compatibility and deployment readiness

## Usage

### Creating Judges with CoreML

```rust
use std::sync::Arc;
use std::path::PathBuf;
use engine_coreml::CoreMLEngine;
use agent_agency_contracts::EngineCaps;
use agent_constitutional_council::{Judges, CouncilCoordinator};

// Create CoreML engine
let model_path = PathBuf::from("/path/to/mistral-model.mlpackage");
let caps = EngineCaps {
    model_id: "mistral-7b-instruct".to_string(),
    max_tokens: 4096,
    supports_json: true,
    supports_structured_output: true,
};

let engine = Arc::new(CoreMLEngine::new(model_path, caps).await?);

// Create all four judges with shared engine
let judges = Judges::new(engine.clone());

// Create council coordinator
let mut council = CouncilCoordinator::new(engine, judges);
```

### Judge Execution Flow

1. **Deterministic Checks**: Each judge runs deterministic CAWS invariant checks first
2. **Critical Violations**: Non-waivable violations cause immediate rejection
3. **LLM Analysis**: For gray-zone decisions, judges use CoreML inference via `JudgeEngine`
4. **Verdict Merging**: Deterministic findings are merged with LLM verdict
5. **Caching**: The `CoreMLEngine` includes prompt caching to avoid redundant inference

## Debate Protocol

The debate protocol is implemented in `agent-orchestration/src/council.rs`:
- Multiple worker models can propose competing solutions
- Judges evaluate solutions using CAWS scoring formula
- Highest-scoring solution is selected based on:
  - Evidence Completeness (30%)
  - Budget Adherence (25%)
  - Gate Integrity (25%)
  - Provenance Clarity (20%)

## Performance Characteristics

- **Prompt Caching**: CoreML engine caches prompts using Blake3 hashing
- **ANE Acceleration**: Automatic Neural Engine acceleration when available
- **Concurrent Execution**: All four judges execute concurrently for faster reviews
- **Token Limits**: Each judge uses appropriate token limits (256 tokens default)

## Integration Points

When integrating with `agent-orchestration`:

1. Create `CoreMLEngine` instance
2. Create `Judges` with `Judges::new(engine.clone())`
3. Create `CouncilCoordinator<CoreMLEngine>` 
4. Wrap in `CouncilCoordinatorAdapter` to implement contracts trait

Note: The `agent-constitutional-council` dependency is currently commented out in `agent-orchestration/Cargo.toml` due to circular dependencies. When uncommented, use the adapter pattern to integrate.

## Testing

Tests use mock engines for fast execution. To test with real CoreML:

```rust
let engine = Arc::new(CoreMLEngine::new(model_path, caps).await?);
let judges = Judges::new(engine.clone());
let mut council = CouncilCoordinator::new(engine, judges);
```

## Author

@darianrosebrook

