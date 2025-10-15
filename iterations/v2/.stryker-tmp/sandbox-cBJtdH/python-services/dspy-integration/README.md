/\*\*

- DSPy Integration Service for Agent Agency V2
-
- @author @darianrosebrook
  \*/

# DSPy Integration Service

Python-based service that provides DSPy capabilities to the Agent Agency V2 TypeScript application.

## Overview

This service implements DSPy's signature-based programming and automatic prompt optimization for:

1. **Rubric Engineering** - Self-optimizing reward computation
2. **Model-Based Judges** - Self-improving evaluation prompts
3. **Recursive Reasoning** - Multi-stage agent reasoning pipelines

## Architecture

```
TypeScript Application (Node.js)
    ↕️ (REST API)
DSPy Service (Python/FastAPI)
    ↕️ (DSPy SDK)
LLM Providers (OpenAI, Anthropic, etc.)
```

## Setup

### Prerequisites

- Python 3.10 or higher
- Node.js 20.5.0 or higher (for main application)

### Installation

```bash
# Create virtual environment
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install dependencies
pip install -r requirements.txt
```

### Environment Variables

Create a `.env` file:

```env
# LLM Provider Configuration
OPENAI_API_KEY=your_openai_key
ANTHROPIC_API_KEY=your_anthropic_key

# Service Configuration
DSPY_SERVICE_HOST=localhost
DSPY_SERVICE_PORT=8001
DSPY_LOG_LEVEL=INFO

# DSPy Configuration
DSPY_DEFAULT_LM=gpt-4-turbo-preview
DSPY_CACHE_DIR=./.dspy_cache
```

## API Endpoints

### Rubric Optimization

```http
POST /api/v1/rubric/optimize
Content-Type: application/json

{
  "task_context": "Agent task description",
  "agent_output": "Generated output",
  "evaluation_criteria": "Criteria to evaluate"
}
```

### Model Judge Evaluation

```http
POST /api/v1/judge/evaluate
Content-Type: application/json

{
  "judge_type": "relevance",
  "artifact": "Output to evaluate",
  "ground_truth": "Expected output",
  "context": "Task context"
}
```

### Signature Optimization

```http
POST /api/v1/optimize/signature
Content-Type: application/json

{
  "signature_id": "rubric_optimization_v1",
  "eval_data": [...],
  "optimizer": "MIPROv2"
}
```

## Development

### Running the Service

```bash
# Development mode with auto-reload
uvicorn main:app --reload --port 8001

# Production mode
uvicorn main:app --workers 4 --port 8001
```

### Running Tests

```bash
# All tests
pytest

# With coverage
pytest --cov=. --cov-report=html

# Specific test file
pytest tests/test_rubric_optimizer.py -v
```

### Type Checking

```bash
mypy . --strict
```

## Integration with TypeScript Application

The TypeScript application communicates with this service via REST API:

```typescript
// src/dspy-integration/DSPyClient.ts
const client = new DSPyClient({ baseUrl: "http://localhost:8001" });

const result = await client.optimizeRubric({
  taskContext: "...",
  agentOutput: "...",
  evaluationCriteria: "...",
});
```

## Phase 1 Implementation Status

- [x] Service structure and dependencies
- [ ] Core DSPy signature implementations
- [ ] FastAPI server with REST endpoints
- [ ] TypeScript client library
- [ ] Integration tests
- [ ] Production deployment configuration

## References

- [DSPy Documentation](https://dspy-docs.vercel.app/)
- [Agent Agency V2 - DSPy Integration Evaluation](../../docs/3-agent-rl-training/dspy-integration-evaluation.md)
- [Integration Decisions Document](../../docs/3-agent-rl-training/INTEGRATION_DECISIONS.md)
