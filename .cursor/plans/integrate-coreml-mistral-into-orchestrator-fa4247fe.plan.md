<!-- fa4247fe-ea25-4bd1-a722-ea08b649d7e4 f07f477d-5fb7-4d6d-886e-92496e443518 -->
# Integrate CoreML Mistral Inference into Orchestrator

## Current State

- `agent-orchestration` CoreML manager has `load_mistral_model()` implemented but models stored as `Arc<MistralModel>` (needs `Arc<Mutex<>>` for mutable inference)
- `PlanGenerator::enhance_with_ai_assistance()` exists but doesn't call any inference
- `system-acceleration` provides `generate_text()` for general Mistral inference
- `engine-coreml` shows the pattern: `Arc<tokio::sync::Mutex<MistralModel>>` for thread-safe inference

## Implementation Plan

### Step 1: Fix CoreML Manager Model Storage

**File**: `iterations/v3/agent-orchestration/src/coreml/mod.rs`

- Change `CoreMLModel.mistral_model` from `Option<Arc<MistralModel>>` to `Option<Arc<tokio::sync::Mutex<MistralModel>>>`
- Update `load_mistral_model()` to wrap model in `Arc<tokio::sync::Mutex<>>` (line 212)
- Update `get_mistral_model()` return type to `Option<Arc<tokio::sync::Mutex<MistralModel>>>` (line 370)
- Update all references to use the mutex pattern

### Step 2: Add General Text Generation Method

**File**: `iterations/v3/agent-orchestration/src/coreml/mod.rs`

- Add `generate_text()` method that:
- Takes prompt string and `MistralInferenceOptions`
- Gets Mistral model via `get_mistral_model()`
- Locks the mutex and calls `system_acceleration::ane::infer::mistral::generate_text()`
- Returns generated text string
- Handle errors gracefully (model not loaded, inference failures)

### Step 3: Integrate into PlanGenerator

**File**: `iterations/v3/agent-orchestration/src/planning/plan_generator.rs`

- Add `CoreMLManager` field to `PlanGenerator` struct (line 30)
- Update `PlanGenerator::new()` to accept optional `CoreMLManager`
- In `enhance_with_ai_assistance()` (line 195), add AI inference calls:
- Call `generate_text()` with planning prompt for milestone decomposition
- Parse AI response to extract milestone suggestions
- Use AI suggestions to enhance milestone creation
- In `decompose_into_milestones()` (line 299), use AI to suggest optimal milestone breakdown
- In `create_milestone_from_criterion()` (line 329), optionally use AI to refine milestone objectives

### Step 4: Initialize CoreML Manager in Orchestrator

**File**: `iterations/v3/agent-orchestration/src/lib.rs` or wherever orchestrator is initialized

- Create `CoreMLManager` instance with model path from `COREML_MODELS_PATH` env var
- Call `load_available_models().await` to load Mistral model
- Pass `CoreMLManager` to `PlanGenerator::new()`
- Handle initialization errors (model not found, load failures)

### Step 5: Add Planning Prompt Templates

**File**: `iterations/v3/agent-orchestration/src/planning/plan_generator.rs` (or new `planning_prompts.rs`)

- Create prompt templates for:
- Milestone decomposition: "Break down this task into optimal milestones..."
- Milestone refinement: "Refine this milestone objective..."
- Dependency analysis: "Analyze dependencies between these milestones..."
- Format prompts with task context, acceptance criteria, constraints

### Step 6: Update Dependencies

**File**: `iterations/v3/agent-orchestration/Cargo.toml`

- Verify `system-acceleration` dependency exists (already present, line 85)
- Ensure `tokio::sync::Mutex` is available (via tokio workspace dependency)

## Integration Points

1. **Model Loading**: `CoreMLManager::load_mistral_model()` - already implemented, needs mutex wrapper
2. **Inference**: `CoreMLManager::generate_text()` - new method wrapping `system_acceleration::ane::infer::mistral::generate_text()`
3. **Planning**: `PlanGenerator::enhance_with_ai_assistance()` - add AI calls for milestone decomposition
4. **Initialization**: Orchestrator startup - load CoreML models and pass to PlanGenerator

## Error Handling

- Model not loaded: Return clear error, fall back to non-AI planning
- Inference timeout: Log warning, use fallback planning
- Parse errors: Log error, use original milestone structure
- Model path missing: Log warning, continue without CoreML

## Testing Considerations

- Test with model loaded and not loaded
- Test inference timeout handling
- Test prompt formatting and parsing
- Verify thread safety with concurrent planning requests

## Files to Modify

1. `iterations/v3/agent-orchestration/src/coreml/mod.rs` - Fix model storage, add generate_text()
2. `iterations/v3/agent-orchestration/src/planning/plan_generator.rs` - Integrate AI inference
3. `iterations/v3/agent-orchestration/src/lib.rs` - Initialize CoreML manager (if orchestrator init is here)
4. Potentially orchestrator initialization code (need to find where PlanGenerator is created)

## Dependencies

- `system-acceleration` crate (already dependency)
- `tokio::sync::Mutex` (via tokio workspace)
- `MistralInferenceOptions` from `system_acceleration::ane::infer::mistral`
- `generate_text()` from `system_acceleration::ane::infer::mistral`

### To-dos

- [ ] Change CoreMLModel.mistral_model to Arc<tokio::sync::Mutex<MistralModel>> and update load_mistral_model() to wrap model in mutex
- [ ] Add generate_text() method to CoreMLManager that locks mutex and calls system_acceleration::ane::infer::mistral::generate_text()
- [ ] Add CoreMLManager field to PlanGenerator and integrate generate_text() calls into enhance_with_ai_assistance() and decompose_into_milestones()
- [ ] Create planning prompt templates for milestone decomposition, refinement, and dependency analysis
- [ ] Find orchestrator initialization code and add CoreMLManager creation/loading, pass to PlanGenerator::new()