# CoreML-First Architecture Decision

## Context

During v3 implementation, we identified architectural drift between documentation and code. This document captures the decision to adopt CoreML-first architecture and the rationale for this choice.

## Decision: CoreML-First Architecture

**Date**: January 2025
**Status**: Implemented (Phase 1-7 in progress)

### Core Tenets

1. **CoreML-first for critical paths**: Constitutional judges, arbitration, and orchestration use CoreML Mistral
2. **ANE acceleration**: Leverage Apple Neural Engine for 2.8x+ inference speedup
3. **Ollama removal**: Complete removal of Ollama dependencies (25 files affected)
4. **Local-first execution**: All critical inference runs on-device via CoreML

### Rationale

#### Performance Benefits

- **ANE Acceleration**: CoreML Mistral targets 2.8x speedup vs CPU fallback (M1) to 3.2x (M3)
- **Lower Latency**: ANE inference expected <50ms for judge deliberations (target)
- **Unified Memory**: Apple Silicon unified memory reduces memory transfer overhead
- **Native Integration**: Direct CoreML APIs eliminate HTTP layer and serialization costs

#### Simplification Benefits

- **Single Model Stack**: CoreML Mistral handles all constitutional reasoning tasks
- **Reduced Dependencies**: Eliminate Ollama runtime HTTP overhead and management complexity
- **Consistent Interface**: Single CoreML interface vs multiple backends to maintain
- **Deployment Simplicity**: CoreML models deploy as optimized bundles with hardware-specific compilation

#### Technical Advantages

- **Optimized Models**: CoreML-optimized Mistral models (7.5 MB FastViT T8 F16 size)
- **Hardware-Specific**: Models compiled for specific Apple Silicon generations (M1, M2, M3, M4)
- **Production Proven**: CoreML proven in production deployments (Kokoro TTS optimization)
- **Memory Efficient**: CoreML models use hardware-optimized memory layouts

### Implementation Status

- ✅ **CoreML Engine**: `engine-coreml` with Mistral loading infrastructure complete
- ✅ **ANE Acceleration**: Hardware acceleration infrastructure operational with CoreML inference
- ✅ **Real Inference**: CoreML Mistral inference enabled and functional
- ✅ **Ollama Removal**: Ollama references removed from production code (deprecated in embedding providers)
- ✅ **Type Migration**: All orchestration types migrated to `agent-agency-contracts`
- ✅ **Memory Integration**: Memory system integrated into autonomous executor
- ✅ **Council Integration**: All judge types use CoreML Mistral inference
- ⚠️ **Embedding Migration**: CoreML-based embeddings planned (Ollama providers deprecated, pending CoreML implementation)
- ⚠️ **Evaluation Framework**: TypeScript evaluation framework port to Rust planned

### Migration Path

See: `iterations/v3/docs/implementation-plan.md` (CoreML-First Orchestration)

1. ✅ **Phase 1**: Enable CoreML real inference - **COMPLETE**
2. ✅ **Phase 2**: Remove dependency compilation errors - **COMPLETE**
3. ✅ **Phase 3**: Complete Ollama removal from production code - **COMPLETE** (deprecated, CoreML-first)
4. ⚠️ **Phase 4**: Port evaluation framework to Rust - **IN PROGRESS**
5. ✅ **Phase 5**: Integrate long-horizon task support - **COMPLETE**
6. ✅ **Phase 6**: Complete autonomous self-prompting loop - **COMPLETE**
7. ✅ **Phase 7**: Finalize council integration - **COMPLETE**

### Backward Compatibility

The CoreML-first architecture maintains model-agnostic interfaces for future extensibility while optimizing for Apple Silicon performance. Hot-swapping capability is preserved for testing and potential future model additions.

## Related Decisions

- **Ollama Removal Strategy**: Ollama dependencies removed from production code. Embedding providers deprecated, pending CoreML implementation
- **Embedding Migration**: Move from deprecated Ollama embeddings to CoreML-based embeddings (planned)
- **Evaluation Framework**: Port TypeScript evaluation to Rust for consistency (planned)