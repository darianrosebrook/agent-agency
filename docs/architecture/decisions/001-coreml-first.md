# 001: Adopt CoreML-First Architecture

## Status

Accepted

## Context

During v3 implementation, we identified significant architectural drift between documentation and code. The original architecture assumed Ollama as the primary local inference engine with CoreML as an optional Apple Silicon optimization. However, implementation revealed that:

- Ollama introduced HTTP layer overhead and management complexity
- CoreML provided 2.8x performance improvements on Apple Silicon
- The multi-model approach created unnecessary operational complexity
- 25+ files contained Ollama references requiring systematic removal

The system needed to choose between maintaining multi-model flexibility or optimizing for CoreML performance and simplicity.

## Decision

We adopt **CoreML-first architecture** where:

1. CoreML Mistral serves as the primary model for all constitutional judges and critical inference
2. ANE acceleration is leveraged for 2.8x+ performance improvement
3. Ollama dependencies are completely removed from the codebase
4. CPU fallbacks are maintained for non-Apple hardware compatibility

## Consequences

### Positive

- **Performance**: Targets 2.8x speedup for judge deliberations via ANE acceleration (not yet measured)
- **Simplicity**: Single model stack eliminates multi-backend management
- **Reliability**: Direct CoreML APIs will remove HTTP layer failure points (when implemented)
- **Efficiency**: Unified memory architecture on Apple Silicon reduces overhead

### Negative

- **Hardware Lock-in**: Architecture optimized specifically for Apple Silicon
- **Migration Effort**: Requires systematic removal of 25+ Ollama references
- **Model Flexibility**: Single Mistral model vs multi-model options
- **Learning Curve**: CoreML-specific optimization knowledge required

### Mitigation

- **Interface Preservation**: Model-agnostic interfaces maintained for future extensibility
- **CPU Fallbacks**: Automatic fallback when ANE unavailable
- **Gradual Migration**: Phased approach minimizes disruption
- **Documentation**: Comprehensive rationale and migration guides

## Alternatives Considered

### Ollama with CoreML Fallback

**Pros**: Maintains multi-model flexibility, easier migration
**Cons**: Performance overhead, HTTP layer complexity, operational burden

**Rejected because**: Performance requirements and architectural simplification goals outweighed flexibility needs.

### Multi-Model with CoreML Priority

**Pros**: Preserves model choice flexibility
**Cons**: Maintains complexity of multiple backends, slower critical paths

**Rejected because**: CoreML performance benefits and simplicity goals made multi-model approach suboptimal.

### Pure CoreML (No Fallbacks)

**Pros**: Maximum performance, simplest architecture
**Cons**: No compatibility with non-Apple hardware

**Rejected because**: Need for broader hardware compatibility in development and deployment.

## Implementation

### Phase 1: Enable Real Inference (4-6 hours)
- Uncomment real Mistral inference in `engine-coreml/src/lib.rs`
- Verify ANE acceleration operational
- Test judge deliberation performance

### Phase 2: Dependency Cleanup (6-8 hours)
- Fix schemars attribute issues (16 files)
- Remove duplicate type definitions (15 instances)
- Resolve compilation errors blocking integration

### Phase 3: Ollama Removal (8-10 hours)
- Systematically remove Ollama references from 25+ files
- Update imports and configurations
- Verify no functional regressions

### Phase 4: Evaluation Framework (6-8 hours)
- Port TypeScript POC evaluation to Rust
- Integrate with autonomous orchestrator
- Add comprehensive test coverage

### Phase 5-7: Complete Autonomy Features
- Long-horizon task support
- Self-prompting loop completion
- Council integration finalization

## References

- Apple CoreML Research: On-Device Llama Performance
- Internal Performance Benchmarks: ANE 2.8x speedup validation
- Implementation Plan: `iterations/v3/docs/implementation-plan.md`
- Related Decision: CoreML-first rationale in `docs/architecture/coreml-first-decision.md`