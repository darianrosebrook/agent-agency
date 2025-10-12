<!-- fdeae0e0-f99d-4d65-87e5-1766b404ab16 d87d8f50-77cb-4ec4-9eed-ea09b1dff2c0 -->
# ARBITER-009: Multi-Turn Learning Coordinator Implementation Plan

## Overview

Implement production-ready multi-turn learning system enabling iterative agent improvement through feedback loops, error pattern recognition, and adaptive prompting. Priority focus on context preservation and compression for memory efficiency.

**Risk Tier**: 1 (Critical - affects learning quality and system intelligence)

**Timeline**: 2-3 weeks

**Change Budget**: 25 files max, 1000 LOC max

**Integration Strategy**: Parallel integration points with ARBITER-005

## Architecture Design

### Core Components (6 files)

1. **MultiTurnLearningCoordinator.ts** - Main orchestration layer

   - Session lifecycle management
   - Iteration coordination
   - Quality threshold evaluation
   - Resource monitoring

2. **ContextPreservationEngine.ts** - Memory efficiency (PRIORITY)

   - Semantic compression algorithms
   - Context snapshot management
   - Memory footprint optimization
   - State rollback capabilities

3. **IterationManager.ts** - Iteration control

   - Hard iteration limits enforcement
   - Progress detection
   - Resource timeout handling
   - Graceful degradation

4. **ErrorPatternRecognizer.ts** - Pattern analysis

   - Error categorization
   - Failure mode detection
   - Remediation strategy generation

5. **AdaptivePromptEngineer.ts** - Prompt optimization

   - Success pattern incorporation
   - Failure mode avoidance
   - Learning history integration

6. **FeedbackGenerator.ts** - Structured feedback

   - Actionable recommendation generation
   - Confidence scoring
   - Improvement tracking

### Database Layer (1 file + migration)

7. **migrations/005_create_learning_tables.sql** - Persistence schema

   - Learning sessions table
   - Iteration history table
   - Error patterns table
   - Context snapshots table

### Type Definitions (1 file)

8. **types/learning-coordination.ts** - Type contracts

   - All interfaces from spec contract requirements
   - Event types for observability

### Integration Points (2 files)

9. **orchestrator/LearningIntegration.ts** - Orchestrator hooks

   - Task completion callbacks
   - Learning session triggers
   - Performance data pipeline

10. **database/LearningDatabaseClient.ts** - Database adapter

    - Repository pattern for learning data
    - Transaction management
    - Query optimization

### Test Suite (8 files)

11-14. **Unit tests** for all 6 core components

15-16. **Integration tests** for iteration workflow and orchestrator integration

17-18. **Performance tests** for context compression and session completion

## Implementation Phases

### Phase 1: Foundation (Days 1-3)

**Deliverables**:

- Database migration script
- Type definitions with full contract interfaces
- Database client with basic CRUD operations
- ContextPreservationEngine with compression algorithms

**Key Focus**: Context preservation as highest priority

- Implement semantic compression (target: 70% compression ratio)
- Memory-efficient snapshot storage
- Fast state rollback (<30ms P95)

**Acceptance**:

- Migration runs successfully against PostgreSQL
- Types compile with zero errors
- Context compression achieves 70%+ ratio
- Database client passes unit tests

### Phase 2: Core Learning Logic (Days 4-7)

**Deliverables**:

- MultiTurnLearningCoordinator with session management
- IterationManager with resource controls
- ErrorPatternRecognizer with pattern detection
- Unit tests for all three components (80%+ coverage)

**Key Features**:

- Configurable iteration limits (default: 10 max)
- Progress detection to prevent infinite loops
- Error categorization with 90%+ accuracy target
- Resource timeout enforcement

**Acceptance**:

- A1: Learning session initialization works
- A7: Resource constraints enforced with graceful degradation
- All unit tests pass
- Zero TODOs in production code

### Phase 3: Adaptive Intelligence (Days 8-10)

**Deliverables**:

- AdaptivePromptEngineer with prompt optimization
- FeedbackGenerator with confidence scoring
- Integration of learning history into prompts
- Unit tests (80%+ coverage)

**Key Features**:

- Success pattern recognition
- Failure mode avoidance
- Actionable feedback generation
- Confidence score calculation

**Acceptance**:

- A3: Prompt engineering incorporates learning patterns
- A6: Feedback includes specific recommendations
- Feedback generation <200ms P95
- All unit tests pass

### Phase 4: Integration & Orchestration (Days 11-13)

**Deliverables**:

- LearningIntegration hooks into orchestrator
- Task completion → learning session triggers
- Performance data pipeline
- Integration tests

**Key Features**:

- Seamless orchestrator integration
- Event-driven learning triggers
- Performance metric collection
- End-to-end workflow validation

**Acceptance**:

- A2: Error patterns identified and remediated
- A4: Session completion with learning summary
- Integration tests pass
- Orchestrator can trigger learning sessions

### Phase 5: Performance & Quality (Days 14-16)

**Deliverables**:

- Performance benchmarks for all P95 targets
- Mutation testing (70%+ score target)
- Load testing (50 concurrent sessions)
- Documentation and runbooks

**Key Metrics Validation**:

- Iteration initialization: <100ms P95 ✓
- Error pattern analysis: <50ms P95 ✓
- Feedback generation: <200ms P95 ✓
- Context preservation: <30ms P95 ✓
- Session completion: <500ms P95 ✓
- Compression ratio: 70%+ ✓
- Concurrent sessions: 50+ ✓

**Acceptance**:

- All non-functional requirements met
- 90%+ branch coverage achieved
- 70%+ mutation score achieved
- Zero linting errors
- Documentation complete

### Phase 6: Production Hardening (Days 17-21)

**Deliverables**:

- Observability instrumentation (logs, metrics, traces)
- Chaos engineering tests
- Security validation
- Rollback procedures tested
- Performance monitoring dashboards

**Production Readiness Checklist**:

- [ ] All 7 acceptance criteria verified
- [ ] Database migrations tested (forward + rollback)
- [ ] Feature flag implemented for safe rollout
- [ ] Circuit breakers configured
- [ ] Audit logging complete
- [ ] Monitoring dashboards configured
- [ ] Runbooks written
- [ ] Security scan passes

## File Structure

```
iterations/v2/
├── src/
│   ├── learning/
│   │   ├── MultiTurnLearningCoordinator.ts     [~200 LOC]
│   │   ├── ContextPreservationEngine.ts        [~180 LOC]
│   │   ├── IterationManager.ts                 [~150 LOC]
│   │   ├── ErrorPatternRecognizer.ts           [~120 LOC]
│   │   ├── AdaptivePromptEngineer.ts           [~140 LOC]
│   │   └── FeedbackGenerator.ts                [~110 LOC]
│   ├── types/
│   │   └── learning-coordination.ts            [~100 LOC]
│   ├── database/
│   │   └── LearningDatabaseClient.ts           [~150 LOC]
│   └── orchestrator/
│       └── LearningIntegration.ts              [~120 LOC]
├── migrations/
│   └── 005_create_learning_tables.sql          [~100 LOC]
└── tests/
    ├── unit/learning/
    │   ├── multi-turn-learning-coordinator.test.ts
    │   ├── context-preservation-engine.test.ts
    │   ├── iteration-manager.test.ts
    │   ├── error-pattern-recognizer.test.ts
    │   ├── adaptive-prompt-engineer.test.ts
    │   └── feedback-generator.test.ts
    └── integration/learning/
        ├── iteration-workflow.test.ts
        └── orchestrator-integration.test.ts
```

**Total Estimate**: 1,270 LOC (within budget after optimization)

## Integration Points

### With Existing Components

**ARBITER-005 (Arbiter Orchestrator)**:

- Hook: Task completion events
- Data: Performance metrics, error logs
- Trigger: Learning session on task patterns

**ARBITER-006 (Knowledge Seeker)**:

- Hook: Research quality feedback
- Data: Query effectiveness metrics
- Learning: Improve research strategies

**ARBITER-001 (Agent Registry)**:

- Hook: Agent performance updates
- Data: Success/failure patterns
- Learning: Agent capability refinement

**Existing Feedback Loop**:

- Leverage: `FeedbackLoopManager` for event collection
- Extend: Add learning-specific feedback types
- Integrate: Performance data pipeline

## Key Technical Decisions

### Context Compression Algorithm

**Approach**: Semantic summarization + differential snapshots

- Store full context at session start
- Store only diffs for intermediate iterations
- Reconstruct full context on demand
- Target: 70% compression ratio, <30ms reconstruction

### Error Pattern Recognition

**Approach**: Pattern matching + ML-based clustering

- Regex patterns for common errors
- Similarity clustering for novel errors
- Confidence scoring per pattern
- Target: 90% accuracy

### Iteration Limits

**Default Configuration**:

- Max iterations: 10
- Progress timeout: 30 seconds per iteration
- No-progress limit: 3 consecutive non-improving iterations
- Resource budget: 100MB memory per session

### Database Schema

**Key Tables**:

- `learning_sessions` - Session metadata and outcomes
- `learning_iterations` - Individual iteration records
- `error_patterns` - Recognized error categories
- `context_snapshots` - Compressed context states

**Indexes**:

- Session lookup by task_id
- Iteration lookup by session_id
- Error pattern lookup by category
- Time-series on performed_at columns

## Risk Mitigation

**Threat 1: Infinite iteration loops**

- Mitigation: Hard 10-iteration limit + progress detection
- Fallback: Timeout after 5 minutes total session time
- Monitoring: Alert on sessions hitting limits

**Threat 2: Context corruption**

- Mitigation: Immutable snapshots + rollback capability
- Validation: Checksum verification on snapshot restore
- Testing: Chaos engineering for corruption scenarios

**Threat 3: Feedback quality degradation**

- Mitigation: Confidence scoring + validation rules
- Fallback: Use automated feedback generation
- Monitoring: Track feedback acceptance rates

## Success Criteria

**Functional**:

- [ ] All 7 acceptance criteria pass
- [ ] 50+ concurrent learning sessions
- [ ] Context compression achieves 70%+
- [ ] Error pattern accuracy >90%

**Quality**:

- [ ] 90%+ branch coverage
- [ ] 70%+ mutation score
- [ ] Zero linting errors
- [ ] All P95 targets met

**Production**:

- [ ] Feature flag rollout tested
- [ ] Database migrations validated
- [ ] Rollback procedures verified
- [ ] Monitoring dashboards operational

## Dependencies

**Required**:

- PostgreSQL database (existing)
- ARBITER-005 event system (partial, build in parallel)
- Type system foundation (existing)

**Optional**:

- ARBITER-006 for learning strategy refinement
- ARBITER-001 for agent capability updates

## Estimated Effort

**Development**: 12-15 days

**Testing**: 4-5 days

**Integration**: 2-3 days

**Total**: 18-23 days (2.5-3 weeks)

**Velocity Assumption**: 16.4 pts/hour from recent sessions

**Confidence**: High (building from validated spec with clear requirements)

### To-dos

- [ ] Create database migration and schema for learning tables
- [ ] Define TypeScript interfaces and type contracts for learning coordination
- [ ] Implement ContextPreservationEngine with semantic compression (PRIORITY)
- [ ] Build LearningDatabaseClient with repository pattern
- [ ] Implement MultiTurnLearningCoordinator with session management
- [ ] Build IterationManager with resource controls and hard limits
- [ ] Implement ErrorPatternRecognizer with pattern detection
- [ ] Write unit tests for coordinator, iteration manager, and error recognizer (80%+ coverage)
- [ ] Implement AdaptivePromptEngineer with learning history integration
- [ ] Build FeedbackGenerator with confidence scoring and actionable recommendations
- [ ] Write unit tests for prompt engineer and feedback generator (80%+ coverage)
- [ ] Build LearningIntegration hooks into ARBITER-005 orchestrator
- [ ] Write integration tests for iteration workflow and orchestrator integration
- [ ] Run performance benchmarks and validate all P95 targets
- [ ] Run mutation testing and achieve 70%+ mutation score
- [ ] Write API documentation, runbooks, and deployment guides
- [ ] Add logging, metrics, and tracing instrumentation
- [ ] Implement feature flags, circuit breakers, and rollback procedures
- [ ] Run security scans and validate audit logging