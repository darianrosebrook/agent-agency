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
- ⚠️ **ANE Acceleration**: Hardware acceleration infrastructure exists but real inference disabled
- ❌ **Real Inference**: Currently returns mock responses (line 267 in `engine-coreml/src/lib.rs` commented)
- ❌ **Ollama References**: 25 files still contain Ollama code pending removal
- ❌ **Evaluation Integration**: POC TypeScript evaluation framework needs Rust port

### Migration Path

See: `iterations/v3/docs/implementation-plan.md` (CoreML-First Orchestration)

1. **Phase 1**: Enable CoreML real inference (4-6 hours)
2. **Phase 2**: Remove dependency compilation errors (6-8 hours)
3. **Phase 3**: Complete Ollama removal from 25 files (8-10 hours)
4. **Phase 4**: Port evaluation framework to Rust (6-8 hours)
5. **Phase 5**: Integrate long-horizon task support (8-10 hours)
6. **Phase 6**: Complete autonomous self-prompting loop (6-8 hours)
7. **Phase 7**: Finalize council integration (4-6 hours)

### Backward Compatibility

The CoreML-first architecture maintains model-agnostic interfaces for future extensibility while optimizing for Apple Silicon performance. Hot-swapping capability is preserved for testing and potential future model additions.

## Related Decisions

- **Ollama Removal Strategy**: Complete removal (not fallback) - see Phase 3 of implementation plan
- **Embedding Migration**: Move from Ollama embeddings to CoreML-based embeddings
- **Evaluation Framework**: Port TypeScript evaluation to Rust for consistency