---

## POC E2E Vision: End-to-End Self-Prompting Agent Tests

### Current POC Components Available for E2E Tests

#### ✅ Already Working:
- **MCP Server**: Full implementation with resource/tool handlers
- **Data Layer**: PostgreSQL + pgvector + Redis with comprehensive monitoring
- **Memory System**: Multi-tenant memory with context offloading and federated learning
- **Agent Orchestrator**: Basic task routing with memory-aware capabilities
- **Database Schema**: Complete migrations with vector support and Row Level Security
- **Unit Tests**: 62/84 tests passing (74% coverage)

#### ❌ Missing for E2E Tests:
1. **Local AI Model Integration** (Gemma 3N/Ollama)
2. **Evaluation Framework** (satisficing logic and quality gates)
3. **E2E Test Runner Infrastructure**
4. **Agent Loop Orchestration**
5. **Test Data and Fixtures**

### E2E Test Scenarios

Based on the three efficient first tests, we'll implement these end-to-end flows:

#### **Test 1: Text Transformation E2E**
**Goal**: Verify agent can rewrite content and self-evaluate completion
- **Input**: Raw paragraph requiring formal rewrite
- **Process**: Agent generates → Evaluates (length, style, banned phrases) → Iterates if needed (max 3x)
- **Success**: Output meets all criteria without over-optimization
- **Validates**: Self-prompting loop, evaluation framework, satisficing logic

#### **Test 2: Code Generation E2E**
**Goal**: Verify agent produces production-quality code
- **Input**: Component specification (e.g., React button with design requirements)
- **Process**: Agent generates → Runs lint/test/typecheck → Fixes issues → Iterates (max 3x)
- **Success**: Code passes all quality gates (tests, lint, types)
- **Validates**: Tool calling, quality gate enforcement, iterative improvement

#### **Test 3: Design Token Application E2E**
**Goal**: Verify agent uses semantic design tokens, not hardcoded values
- **Input**: UI component requirements with token registry
- **Process**: Agent generates → Scans for hardcoded values → Replaces with tokens → Iterates (max 3x)
- **Success**: No hex colors, no raw px spacing, proper token usage
- **Validates**: Design system compliance, token awareness, semantic coding

### Implementation Phases

#### **Phase 1: Core Infrastructure (1-2 weeks)**
1. **Add Ollama integration** to POC MCP server
2. **Implement evaluation framework** from agent-agency.md
3. **Create test fixtures** and sample data
4. **Set up E2E test runner**

#### **Phase 2: Basic E2E Tests (1 week)**
1. **Text transformation test** (simplest to implement)
2. **Basic MCP flow validation**
3. **Evaluation framework testing**

#### **Phase 3: Advanced E2E Tests (1-2 weeks)**
1. **Code generation test** (requires linting setup)
2. **Design token test** (requires token registry)
3. **Multi-iteration scenarios**
4. **Performance and reliability tests**

### Success Criteria for E2E Tests

#### **Test Validation:**
- ✅ **Text transformation**: Agent can rewrite content and recognize completion
- ✅ **Code generation**: Agent produces lint-clean, tested code
- ✅ **Token application**: Agent uses semantic tokens, not hardcoded values
- ✅ **Self-evaluation**: Agent stops iterating when quality thresholds met
- ✅ **Performance**: Tests complete within 2 minutes each
- ✅ **Reliability**: 95%+ test pass rate in CI

#### **Infrastructure Validation:**
- ✅ **Model integration**: Gemma 3N responds reliably
- ✅ **MCP protocol**: Full request/response cycle works
- ✅ **Data persistence**: Agent memory and results persist
- ✅ **Evaluation framework**: All 3 evaluator types working

### Required Dependencies & Setup

#### **New Dependencies**
```json
{
  "ollama": "^0.3.0",
  "openai": "^4.24.7",
  "@playwright/test": "^1.40.0",
  "testcontainers": "^10.0.0"
}
```

#### **Environment Setup**
```bash
# Start required services for E2E tests
docker run -d -p 5432:5432 postgres:16
docker run -d -p 6379:6379 redis:7
ollama serve  # Start Ollama server
ollama pull gemma:3n  # Pull the model
```

#### **Test Scripts**
```json
{
  "test:e2e": "playwright test tests/e2e/",
  "test:e2e:setup": "node scripts/setup-e2e-env.js",
  "test:e2e:ci": "npm run test:e2e:setup && npm run test:e2e"
}
```

### Next Steps Implementation

1. **Start with evaluation framework** - Implement the scripts from agent-agency.md
2. **Add Ollama client** - Integrate local model calling
3. **Create test fixtures** - Sample inputs for each test type
4. **Build E2E runner** - Test orchestration framework
5. **Implement Text E2E test** - Start with the simplest scenario

---
