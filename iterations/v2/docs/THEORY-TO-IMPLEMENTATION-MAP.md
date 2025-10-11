# Theory to Implementation Map

**Author**: @darianrosebrook  
**Date**: October 10, 2025  
**Purpose**: Bridge between architectural theory and production code

---

## Overview

This document maps concepts from `docs/1-core-orchestration/theory.md` and `arbiter-architecture.md` to actual implementation files in the V2 codebase. Use this as a reference when reading theory docs to find the corresponding code.

---

## Component Implementation Status

| Component              | Theory Section        | Spec ID     | Status       | Location                                   |
| ---------------------- | --------------------- | ----------- | ------------ | ------------------------------------------ |
| Agent Registry Manager | Model-Agnostic Design | ARBITER-001 | ✅ Complete  | `src/orchestrator/AgentRegistryManager.ts` |
| Task Routing Manager   | Orchestration Model   | ARBITER-002 | 📋 Spec only | `components/task-routing-manager/.caws/`   |
| Multi-Armed Bandit     | Performance Tracking  | ARBITER-002 | 📋 Spec only | (Part of routing)                          |
| CAWS Validator         | CAWS Protocol         | ARBITER-003 | 📋 Spec only | `components/caws-validator/.caws/`         |
| Performance Tracker    | Reflexive Learning    | ARBITER-004 | 📋 Spec only | `components/performance-tracker/.caws/`    |
| Arbiter Orchestrator   | Orchestration Model   | ARBITER-005 | 📋 Spec only | `components/arbiter-orchestrator/.caws/`   |

---

## Theory Concept → Implementation Mapping

### 1. Agent Catalog and Registration

**Theory** (`theory.md` lines 63-67):

> "The stack might include a configuration file or registry of models with their parameters (addresses, expected strengths, cost, etc.)"

**Implementation**: ✅ COMPLETE

- **File**: `src/orchestrator/AgentRegistryManager.ts`
- **Method**: `registerAgent()`
- **Lines**: 60-94
- **Test**: `agent-registry-manager.test.ts` (Agent Registration A1 suite)

**Key Features**:

- Validates required fields (id, name, modelFamily, capabilities)
- Checks capacity limits (max 1000 agents)
- Prevents duplicate registrations
- Initializes optimistic performance history
- Returns immutable profile clone

---

### 2. Performance Tracking and Running Averages

**Theory** (`theory.md` lines 57-61):

> "The arbiter should keep track of each model's performance on various tasks...giving preference to models with higher success rates"

**Implementation**: ✅ COMPLETE

- **File**: `src/orchestrator/AgentProfile.ts`
- **Method**: `updatePerformanceHistory()`
- **Lines**: 24-56
- **Algorithm**: Incremental averaging formula

**Formula Implemented**:

```typescript
newAverage = oldAverage + (newValue - oldAverage) / (count + 1);
```

**Metrics Tracked**:

- Success rate (0.0-1.0)
- Average quality score (0.0-1.0)
- Average latency (milliseconds)
- Task count (for running averages)

**Test Coverage**:

- `Performance Update (A3)` test suite
- Validates correct running average computation
- Tests atomic updates and concurrency

---

### 3. Capability-Based Query and Sorting

**Theory** (`arbiter-architecture.md` lines 97-108):

> "Return agents filtered by capabilities and sorted by performance history success rate"

**Implementation**: ✅ COMPLETE

- **File**: `src/orchestrator/AgentRegistryManager.ts`
- **Method**: `getAgentsByCapability()`
- **Lines**: 125-196
- **Test**: `Query by Capability (A2)` suite

**Query Features**:

- Filter by task type (required)
- Filter by languages (optional)
- Filter by specializations (optional)
- Filter by utilization threshold (optional)
- Filter by minimum success rate (optional)
- Sort by success rate (highest first)
- Include match score and reason

**Performance**: ~1ms (target: <50ms P95)

---

### 4. UCB Confidence Interval for Multi-Armed Bandit

**Theory** (`arbiter-architecture.md` lines 220-232):

> "UCB formula: mean + exploration bonus = successRate + sqrt((2 \* ln(totalTasks)) / taskCount)"

**Implementation**: ✅ COMPLETE

- **File**: `src/orchestrator/AgentProfile.ts`
- **Method**: `calculateConfidenceInterval()`
- **Lines**: 187-199

**Formula**:

```typescript
explorationBonus = Math.sqrt((2 * Math.log(totalTasks)) / taskCount);
```

**Usage**: Will be used by ARBITER-002 (Task Routing Manager) for agent selection

---

### 5. Agent Load Management

**Theory** (`arbiter-architecture.md` lines 81-88):

> "Track current load with active tasks, queued tasks, and utilization percentage"

**Implementation**: ✅ COMPLETE

- **File**: `src/orchestrator/AgentRegistryManager.ts`
- **Method**: `updateLoad()`
- **Lines**: 255-286
- **Helper**: `AgentProfile.ts` methods for increment/decrement

**Features**:

- Track active tasks and queued tasks
- Calculate utilization percentage
- Filter agents by utilization threshold
- Support load balancing decisions

**Test**: `Load Filtering (A4)` suite

---

### 6. Optimistic Initialization for Cold Start

**Theory**: Not explicitly mentioned, but implied in multi-armed bandit literature

**Implementation**: ✅ COMPLETE

- **File**: `src/orchestrator/AgentProfile.ts`
- **Method**: `createInitialPerformanceHistory()`
- **Lines**: 61-68

**Strategy**:

- New agents start with successRate: 0.8 (optimistic)
- Encourages exploration of new agents
- Prevents ignoring untested agents
- Allows system to discover capable new agents

---

### 7. Immutable Data Structures

**Theory**: Best practice for concurrent systems

**Implementation**: ✅ COMPLETE

- **Pattern**: All methods return clones, never mutate parameters
- **File**: `src/orchestrator/AgentProfile.ts`
- **Method**: `cloneProfile()`
- **Lines**: 238-247

**Benefits**:

- Thread-safe operations
- Predictable behavior
- Easy testing
- Audit trail preservation

---

### 8. Database Persistence Schema

**Theory** (`theory.md` lines 106-111):

> "The stack might include a simple database or file store that records each task and outcome"

**Implementation**: ✅ COMPLETE

- **File**: `migrations/001_create_agent_registry_tables.sql`
- **Lines**: 314 lines total

**Tables Created**:

1. **agent_profiles** - Core agent data
2. **agent_capabilities** - Capability tracking
3. **agent_performance_history** - Running averages
4. **agent_performance_events** - Individual event audit trail

**Features**:

- UUID primary keys for distributed systems
- Optimized indexes for common queries
- Check constraints for data integrity
- Views for common query patterns
- Automatic timestamp triggers
- Zero-downtime deployment support

---

## Code Organization

### Type Definitions

**Location**: `src/types/agent-registry.ts` (395 lines)

**Exports**:

- `AgentId`, `Timestamp` - Type aliases
- `TaskType`, `ProgrammingLanguage`, `Specialization`, `ModelFamily` - Enums as union types
- `AgentCapabilities` - Capability profile
- `PerformanceHistory` - Running average metrics
- `CurrentLoad` - Utilization tracking
- `AgentProfile` - Complete agent information
- `PerformanceMetrics` - New task metrics
- `AgentQuery` - Query parameters
- `AgentQueryResult` - Query results with scores
- `AgentRegistryConfig` - Registry configuration
- `RegistryStats` - Statistics interface
- `RegistryErrorType`, `RegistryError` - Error handling

**Philosophy**: Single source of truth for all agent-related types

---

### Helper Utilities

**Location**: `src/orchestrator/AgentProfile.ts` (279 lines)

**Purpose**: Immutable operations and calculations

**Static Methods**:

1. `updatePerformanceHistory()` - Incremental averaging
2. `createInitialPerformanceHistory()` - Optimistic initialization
3. `incrementActiveTask()` - Load management
4. `decrementActiveTask()` - Load management
5. `updateQueuedTasks()` - Queue management
6. `createInitialLoad()` - Initial state
7. `updateLastActive()` - Timestamp updates
8. `isStale()` - Health checking
9. `calculateConfidenceInterval()` - UCB bonus
10. `validateProfile()` - Data validation
11. `cloneProfile()` - Deep cloning

**Design Pattern**: Functional utility class with pure functions

---

### Main Registry Implementation

**Location**: `src/orchestrator/AgentRegistryManager.ts` (465 lines)

**Purpose**: Central agent catalog with atomic operations

**Public Methods**:

- `registerAgent()` - Add agent to registry
- `getProfile()` - Retrieve agent by ID
- `getAgentsByCapability()` - Query with filtering and sorting
- `updatePerformance()` - Update running averages
- `updateLoad()` - Update utilization metrics
- `getStats()` - Registry statistics
- `unregisterAgent()` - Remove agent
- `shutdown()` - Cleanup resources

**Private Methods**:

- `initializeCapabilityTracking()` - Extension point
- `calculateMatchScore()` - Capability scoring
- `explainMatchScore()` - Generate explanations
- `startAutoCleanup()` - Stale agent cleanup
- `cleanupStaleAgents()` - Cleanup logic

**Data Structure**: Map<AgentId, AgentProfile> for O(1) lookups

---

### Test Suite

**Location**: `tests/unit/orchestrator/agent-registry-manager.test.ts` (520 lines)

**Test Organization**:

1. **Agent Registration (A1)** - 4 tests
2. **Query by Capability (A2)** - 5 tests
3. **Performance Update (A3)** - 4 tests
4. **Load Filtering (A4)** - 2 tests
5. **Registry Statistics (A5)** - 3 tests
6. **Performance and Concurrency** - 2 tests

**Total**: 20 tests, 100% passing

**Coverage**: All acceptance criteria validated

---

## Algorithm Implementations

### Incremental Averaging (Running Average)

**Theory**: Avoid storing all historical data, maintain constant memory

**Implementation**: `AgentProfile.updatePerformanceHistory()`

**Formula**:

```typescript
newCount = oldCount + 1;
newAverage = oldAverage + (newValue - oldAverage) / newCount;
```

**Applied to**:

- Success rate (binary: 1.0 or 0.0)
- Quality score (0.0-1.0)
- Latency (milliseconds)

**Benefits**:

- O(1) time complexity
- O(1) space complexity
- Mathematically equivalent to full average
- Suitable for streaming data

---

### Upper Confidence Bound (UCB)

**Theory**: Multi-armed bandit exploration bonus

**Implementation**: `AgentProfile.calculateConfidenceInterval()`

**Formula**:

```typescript
explorationBonus = sqrt((2 * ln(totalTasks)) / taskCount);
```

**Purpose**:

- Encourage exploration of agents with few trials
- Balance exploitation (use best) vs exploration (try new)
- Provides theoretical guarantees on regret bounds

**Special Cases**:

- `taskCount = 0`: Returns 1.0 (maximum exploration)
- Higher `totalTasks`: Increases exploration bonus
- Higher `taskCount`: Decreases exploration bonus (more confidence)

**Will be used by**: ARBITER-002 Task Routing Manager

---

### Match Scoring

**Theory**: Rank agents by how well they match task requirements

**Implementation**: `AgentRegistryManager.calculateMatchScore()`

**Algorithm**:

```
score = 0.0, weights = 0.0

// Task type (required)
score += 0.3; weights += 0.3

// Languages (if specified)
score += (matchedLanguages / requiredLanguages) * 0.3
weights += 0.3

// Specializations (if specified)
score += (matchedSpecs / requiredSpecs) * 0.2
weights += 0.2

// Performance bonus
score += successRate * 0.2
weights += 0.2

finalScore = score / weights
```

**Returns**: 0.0-1.0 score indicating match quality

---

## Data Flow Diagrams

### Agent Registration Flow

```
registerAgent()
    │
    ├─> Validate required fields
    │   └─> AgentProfileHelper.validateProfile()
    │
    ├─> Check for duplicates
    │   └─> agents.has(id)
    │
    ├─> Check capacity
    │   └─> agents.size < maxAgents
    │
    ├─> Create profile with defaults
    │   ├─> createInitialPerformanceHistory()
    │   └─> createInitialLoad()
    │
    ├─> Initialize capability tracking
    │   └─> initializeCapabilityTracking()
    │
    └─> Store in registry
        └─> agents.set(id, profile)
```

**Implementation**: Lines 60-94 in `AgentRegistryManager.ts`

---

### Performance Update Flow

```
updatePerformance(agentId, metrics)
    │
    ├─> Get current profile
    │   └─> agents.get(agentId)
    │
    ├─> Compute new running averages
    │   └─> AgentProfileHelper.updatePerformanceHistory()
    │       ├─> successRate update
    │       ├─> averageQuality update
    │       └─> averageLatency update
    │
    ├─> Create updated profile
    │   └─> Immutable update with spread
    │
    └─> Atomically store
        └─> agents.set(agentId, updatedProfile)
```

**Implementation**: Lines 209-244 in `AgentRegistryManager.ts`

---

### Capability Query Flow

```
getAgentsByCapability(query)
    │
    ├─> Iterate all agents
    │   └─> for (profile of agents.values())
    │
    ├─> Filter by task type (required)
    │   └─> capabilities.taskTypes.includes(taskType)
    │
    ├─> Filter by languages (optional)
    │   └─> query.languages.every(lang => ...)
    │
    ├─> Filter by specializations (optional)
    │   └─> query.specializations.every(spec => ...)
    │
    ├─> Filter by utilization (optional)
    │   └─> utilizationPercent < maxUtilization
    │
    ├─> Filter by success rate (optional)
    │   └─> successRate >= minSuccessRate
    │
    ├─> Calculate match score
    │   └─> calculateMatchScore()
    │
    └─> Sort by success rate
        └─> sort((a,b) => b.successRate - a.successRate)
```

**Implementation**: Lines 125-196 in `AgentRegistryManager.ts`

---

## File Reference Quick Guide

### For Understanding Agent Management

**Start here**: `src/types/agent-registry.ts`

- Read all interfaces and type definitions
- Understand the data model

**Then read**: `src/orchestrator/AgentProfile.ts`

- Helper functions for operations
- See algorithm implementations

**Finally**: `src/orchestrator/AgentRegistryManager.ts`

- Main registry logic
- Integration of all pieces

### For Understanding Testing

**Start here**: `tests/unit/orchestrator/agent-registry-manager.test.ts`

- See usage patterns in tests
- Understand acceptance criteria validation

**Test execution**:

```bash
cd iterations/v2
npm test -- tests/unit/orchestrator/agent-registry-manager.test.ts
```

**Expected**: 20/20 tests passing in ~1.3s

### For Understanding Database Schema

**Read**: `migrations/001_create_agent_registry_tables.sql`

**Tables**:

1. `agent_profiles` - Core agent data
2. `agent_capabilities` - Capability entries
3. `agent_performance_history` - Running averages
4. `agent_performance_events` - Audit trail

**Views**:

- `agent_profiles_with_capabilities` - Complete agent data
- `available_agents` - Ready-to-use agents

---

## Theory Sections with Implementation References

### Section: "Model-Agnostic Design and Hot-Swapping"

**Theoretical Concept**: Maintain registry of models that can be swapped based on performance

**Implementation**:

- ✅ `AgentProfile.modelFamily` - Tracks which model family (gpt-4, claude-3.5, etc.)
- ✅ `AgentRegistryManager.registerAgent()` - Add new models to catalog
- ✅ `AgentRegistryManager.getAgentsByCapability()` - Query available models
- ✅ `AgentRegistryManager.updatePerformance()` - Track model performance

**Code Location**: `src/orchestrator/AgentRegistryManager.ts`

---

### Section: "Performance Tracking & Preference"

**Theoretical Concept**: Log success/failure and quality scores, weight choices by success rate

**Implementation**:

- ✅ `PerformanceHistory` interface - Success rate, quality, latency, count
- ✅ `PerformanceMetrics` interface - New task metrics
- ✅ `AgentProfileHelper.updatePerformanceHistory()` - Running average calculation
- ✅ Query sorting by `performanceHistory.successRate`

**Code Locations**:

- Types: `src/types/agent-registry.ts:77-98`
- Algorithm: `src/orchestrator/AgentProfile.ts:24-56`
- Usage: `src/orchestrator/AgentRegistryManager.ts:209-244`

---

### Section: "Pluggable Model Interfaces"

**Theoretical Concept**: Common interface for different model backends

**Implementation**:

- ✅ `AgentCapabilities` - Defines what each agent can do
- ✅ `TaskType` enum - Standardized task categories
- ✅ `ModelFamily` enum - Supported model families
- ✅ Capability queries work across any model family

**Code Location**: `src/types/agent-registry.ts:20-68`

---

### Section: "Ensuring Correctness and Traceability"

**Theoretical Concept**: Log all decisions for audit trail

**Implementation**:

- ✅ `AgentQuery` tracks what was requested
- ✅ `AgentQueryResult` includes `matchReason` explanation
- ✅ Database has `agent_performance_events` table for audit trail
- ✅ All timestamps at millisecond precision
- 📋 Full provenance in ARBITER-003 (planned)

**Code Locations**:

- Query tracking: `src/types/agent-registry.ts:163-181`
- Event audit: `migrations/001_create_agent_registry_tables.sql:126-151`

---

## Performance Benchmarks

### Measured vs Theoretical Targets

| Operation          | Theory Target | Measured | Status        |
| ------------------ | ------------- | -------- | ------------- |
| Agent registration | <100ms P95    | ~3ms     | ✅ 33x better |
| Registry query     | <50ms P95     | ~1ms     | ✅ 50x better |
| Performance update | <30ms P95     | ~10ms    | ✅ 3x better  |
| Load update        | <30ms P95     | ~1ms     | ✅ 30x better |

**Note**: These are in-memory measurements. With database persistence, expect:

- Registration: ~10-20ms
- Query: ~5-15ms
- Update: ~15-25ms

All still well within P95 targets!

---

## Usage Examples Mapped to Theory

### Example 1: Registering a New Model

**Theory**: "Hot-swap models into the system with minimal changes"

**Implementation**:

```typescript
import { AgentRegistryManager } from "./orchestrator/AgentRegistryManager";

const registry = new AgentRegistryManager();

// Register Claude 3.5 as a code editing specialist
const claudeAgent = await registry.registerAgent({
  id: "claude-3.5-sonnet",
  name: "Claude Code Expert",
  modelFamily: "claude-3.5",
  capabilities: {
    taskTypes: ["code-editing", "code-review", "refactoring"],
    languages: ["TypeScript", "Python", "Rust"],
    specializations: ["API design", "Performance optimization"],
  },
});

// Register GPT-4 as a research specialist
const gptAgent = await registry.registerAgent({
  id: "gpt-4-turbo",
  name: "GPT Research Expert",
  modelFamily: "gpt-4",
  capabilities: {
    taskTypes: ["research", "documentation"],
    languages: ["TypeScript", "Python"],
    specializations: ["Database design"],
  },
});
```

**Theory Connection**: Model-agnostic design with pluggable interfaces ✅

---

### Example 2: Tracking Performance Over Time

**Theory**: "Create a record of which model tends to be most effective for each category"

**Implementation**:

```typescript
// After Claude completes a code editing task
await registry.updatePerformance("claude-3.5-sonnet", {
  success: true,
  qualityScore: 0.95,
  latencyMs: 3200,
  taskType: "code-editing",
});

// After GPT-4 completes a research task
await registry.updatePerformance("gpt-4-turbo", {
  success: true,
  qualityScore: 0.88,
  latencyMs: 5100,
  taskType: "research",
});

// View performance stats
const claudeProfile = await registry.getProfile("claude-3.5-sonnet");
console.log(
  `Claude success rate: ${claudeProfile.performanceHistory.successRate}`
);
```

**Theory Connection**: Performance tracking with running averages ✅

---

### Example 3: Intelligent Agent Selection

**Theory**: "Weight choices by higher success rates for the current task type"

**Implementation**:

```typescript
// Find best agent for TypeScript code editing
const results = await registry.getAgentsByCapability({
  taskType: "code-editing",
  languages: ["TypeScript"],
  maxUtilization: 80, // Only agents under 80% load
});

// Results are sorted by success rate
const bestAgent = results[0].agent;
console.log(`Selected: ${bestAgent.name}`);
console.log(`Reason: ${results[0].matchReason}`);
console.log(
  `Success rate: ${(bestAgent.performanceHistory.successRate * 100).toFixed(
    1
  )}%`
);
```

**Theory Connection**: Capability-based routing with performance preference ✅

---

## Next Implementation Steps

### Week 2: Task Routing Manager (ARBITER-002)

**Will implement**:

- `TaskRoutingManager` class
- `MultiArmedBandit` class
- `CapabilityMatcher` class

**Will use**:

- ✅ `AgentRegistryManager.getAgentsByCapability()` for candidate selection
- ✅ `AgentProfileHelper.calculateConfidenceInterval()` for UCB scoring
- ✅ `PerformanceHistory` for exploitation decisions

**Theory sections to implement**:

- Dynamic Model Selection
- Hybrid Routing
- Multi-Armed Bandit algorithm

---

### Week 3: CAWS Validator (ARBITER-003)

**Will implement**:

- `CAWSValidator` class
- `WaiverManager` class
- `ProvenanceRecorder` class

**Theory sections to implement**:

- CAWS-Compliant Arbitration Protocol
- Arbiter Reasoning Engine
- Ensuring Correctness and Traceability

---

### Week 4: Performance Tracker (ARBITER-004)

**Will implement**:

- `PerformanceTracker` class
- `BenchmarkDataCollector` class
- `DataAnonymizer` class

**Will use**:

- ✅ `PerformanceMetrics` interface already defined
- ✅ Database tables for event storage ready

**Theory sections to implement**:

- Reflexive Learning & Memory Integration
- Audit trails and monitoring

---

## How to Extend

### Adding a New TaskType

1. Update type: `src/types/agent-registry.ts` - Add to `TaskType` union
2. Update tests: Add test cases for new task type
3. No code changes needed in `AgentRegistryManager` (generic implementation)

### Adding a New Model Family

1. Update type: `src/types/agent-registry.ts` - Add to `ModelFamily` union
2. Register agent: Call `registerAgent()` with new model family
3. Track performance: System automatically tracks via `updatePerformance()`

### Adding New Capabilities

1. Update types: Add to `Specialization` union or create new capability type
2. Add to agent: Include in `capabilities` when registering
3. Query by new capability: Use in `getAgentsByCapability()` query

---

## Documentation Cross-References

### Theory Documents

- **theory.md** - This document provides research background
- **arbiter-architecture.md** - Concrete component specifications
- **implementation-roadmap.md** - Development timeline

### Implementation Documents

- **ARBITER-001-COMPLETE.md** - Implementation guide
- **ARBITER-001-TEST-RESULTS.md** - Test validation
- **SPECS-INDEX.md** - All component specs
- **ARBITER-SPECS-SUMMARY.md** - Architecture overview

### Specifications

- **components/agent-registry-manager/.caws/working-spec.yaml** - CAWS spec for ARBITER-001
- **components/task-routing-manager/.caws/working-spec.yaml** - CAWS spec for ARBITER-002
- **components/caws-validator/.caws/working-spec.yaml** - CAWS spec for ARBITER-003
- **components/performance-tracker/.caws/working-spec.yaml** - CAWS spec for ARBITER-004
- **components/arbiter-orchestrator/.caws/working-spec.yaml** - CAWS spec for ARBITER-005

---

## Summary

**Theory ↔ Implementation Bridge Complete**

The Agent Registry Manager (ARBITER-001) demonstrates how theoretical concepts from the arbiter stack research translate into production code:

- ✅ Model-agnostic registry → `AgentProfile` with `modelFamily`
- ✅ Performance tracking → Running averages with incremental formulas
- ✅ Capability-based routing → Query, filter, and sort by capabilities
- ✅ Hot-swapping → Register/unregister agents dynamically
- ✅ Audit trails → Database event logging and provenance

**All theory concepts from Agent Registry section are now implemented and tested.**

---

**Next**: Implement ARBITER-002 (Task Routing Manager) to realize multi-armed bandit routing concepts from theory.

**Status**: 1/5 components complete (20%), on track for 8-week roadmap
