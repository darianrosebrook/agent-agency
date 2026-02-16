# Agent Agency V4

A principled AI agent system built on Sterling's operator taxonomy and constitutional governance.

## Current State

**619 tests** across layers (as of 2026-02-12).

| Layer | Crates | Tests | Status |
|-------|--------|-------|--------|
| Core | v4-types, v4-invariants, v4-governance, v4-config | 168 | ✅ |
| Reasoning | v4-symbolic, v4-council, v4-arbiter | 110 | ✅ |
| Infrastructure | v4-storage, v4-postgres, v4-inference, v4-memory, v4-observability | 146 | ✅ |
| Execution | v4-tools, v4-workers, v4-sandbox | 73 | ✅ |
| Interface | v4-api, v4-mcp, v4-a2a | 102 | ✅ |
| Integration | tests/ | 20 | ✅ |

## Quick Start

```bash
# Build
cargo build --workspace

# Test
cargo test

# Run API server
cargo run -p v4-api --bin v4-server
# Server starts on http://127.0.0.1:8080

# Test the API
curl http://localhost:8080/health
curl -X POST http://localhost:8080/api/v1/tasks \
  -H "Content-Type: application/json" \
  -d '{"title": "Test", "description": "Read a file"}'

# Test LLM inference
curl -X POST http://localhost:8080/api/v1/probe \
  -H "Content-Type: application/json" \
  -d '{"prompt": "Explain recursion", "max_tokens": 100}'
```

## Verification

Run the automated smoke test to verify all components work correctly:

```bash
# Full build and test
./scripts/smoke-test.sh

# Skip build (faster for re-testing)
./scripts/smoke-test.sh --skip-build

# Custom port
./scripts/smoke-test.sh --port 9000
```

The smoke test verifies:
- Health endpoint returns "healthy"
- API info endpoint works
- Task submission creates a task with authorization result
- Task status retrieval works
- Chain-of-thought observability endpoint
- Council decisions observability endpoint
- Worker actions observability endpoint
- LLM probe endpoint
- Metrics endpoint

## MCP Server

The MCP (Model Context Protocol) server exposes V4 tools to external clients like Claude Desktop:

```bash
# Run MCP server
cargo run -p v4-mcp --bin v4-mcp-server
# Server starts on http://127.0.0.1:3000

# Test MCP initialization
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}'

# List available tools
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}'
```

### MCP Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/` | POST | JSON-RPC endpoint |
| `/mcp` | POST | JSON-RPC endpoint (alias) |
| `/health` | GET | Health check |
| `/info` | GET | Server capabilities |

### MCP Methods

| Method | Description |
|--------|-------------|
| `initialize` | Initialize connection with client capabilities |
| `initialized` | Notification that initialization is complete |
| `tools/list` | List all available tools |
| `tools/call` | Execute a tool with arguments |
| `ping` | Health check |

## A2A Worker

The A2A (Agent-to-Agent) server exposes an OpenAI-compatible LLM as a peer agent via the [A2A protocol](https://google.github.io/A2A/):

```bash
# Run with MiniMax (default)
MINIMAX_API_KEY=your-key cargo run --bin a2a_worker
# Server starts on http://127.0.0.1:3001

# Run with DeepSeek via OpenRouter
PROVIDER=openrouter OPENROUTER_API_KEY=your-key cargo run --bin a2a_worker

# Run with local Ollama
PROVIDER=ollama OLLAMA_MODEL=llama3.2 cargo run --bin a2a_worker

# Discover the agent
curl http://localhost:3001/.well-known/agent-card.json

# Send work
curl -X POST http://localhost:3001/ \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"message/send","params":{"message":{"messageId":"1","role":"user","parts":[{"kind":"text","text":"Write a haiku about Rust"}],"kind":"message"}}}'
```

### A2A Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/` | POST | JSON-RPC endpoint (message/send, tasks/get, tasks/cancel) |
| `/.well-known/agent-card.json` | GET | Agent discovery card |
| `/stream` | POST | SSE streaming endpoint |
| `/health` | GET | Health check |

## Architecture

```
HTTP POST /api/v1/tasks
        │
        ▼
    v4-api (timing metrics)
        │
        ▼
TaskRequest → v4-symbolic → v4-council → v4-arbiter → v4-workers → v4-sandbox → v4-tools
                  │              │             │
                  │              │             └── Generate VerificationCertificate
                  │              └── 3 Judges: Constitutional, Technical, Quality
                  └── Deterministic operator selection with provenance
```

### Operator Taxonomy (S/M/P/K/C)

| Operator | Purpose | Side Effects |
|----------|---------|--------------|
| **S**eek | Information retrieval | No |
| **M**emorize | Store information | Yes |
| **P**erceive | Interpret input | No |
| **K**nowledge | Apply domain knowledge | No |
| **C**ontrol | Flow control | Yes |

### Security Levels

| Level | Filesystem | Network | Use Case |
|-------|------------|---------|----------|
| Permissive | Full | Yes | Development |
| Standard | /tmp | No | Default |
| Restricted | /tmp (read) | No | Sensitive |
| Strict | None | No | High-risk |

## Crate Structure

```
crates/
├── core/
│   ├── v4-types/         # Shared types, OperatorType, CouncilVerdict
│   ├── v4-config/        # Configuration management
│   ├── v4-invariants/    # Sterling invariants, CAWS constraints
│   └── v4-governance/    # CAWS gates, policy enforcement

├── reasoning/
│   ├── v4-symbolic/      # Operator graphs, rule engine, provenance
│   ├── v4-council/       # 3-judge evaluation, veto logic
│   └── v4-arbiter/       # Final decisions, certificates, routing

├── infrastructure/
│   ├── v4-storage/       # Content-addressable storage, event sourcing
│   ├── v4-postgres/      # PostgreSQL + pgvector for embeddings
│   ├── v4-inference/     # Local LLM inference (Mock, MLX)
│   ├── v4-memory/        # Knowledge graph, Sterling-style decay
│   └── v4-observability/ # Metrics, tracing, health checks

├── execution/
│   ├── v4-tools/         # Tool trait, registry, built-in tools
│   ├── v4-workers/       # Worker pool, task queue, execution
│   └── v4-sandbox/       # Security policies, isolated execution

└── interfaces/
    ├── v4-api/           # HTTP API server with timing metrics
    ├── v4-mcp/           # MCP protocol server for external tool access
    └── v4-a2a/           # A2A protocol server + client for agent-to-agent delegation
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Health check |
| GET | `/metrics` | Timing metrics for LLM probing |
| GET | `/api/v1` | API info |
| POST | `/api/v1/tasks` | Submit task for evaluation |
| GET | `/api/v1/tasks/:id` | Get task status |
| GET | `/api/v1/tasks/:id/chain-of-thought` | Get reasoning steps |
| GET | `/api/v1/tasks/:id/council-decisions` | Get council judge decisions |
| GET | `/api/v1/tasks/:id/worker-actions` | Get worker execution actions |
| POST | `/api/v1/probe` | Probe LLM inference |

## Key Invariants

- **INV-CORE-04**: Deterministic operator selection
- **INV-CORE-05**: Provenance required for decisions
- **INV-CORE-07**: Termination guarantee (bounded iterations)
- **INV-CORE-09**: Fail-closed on uncertainty
- **INV-CORE-10**: Cryptographic audit trail

## Documentation

- [Implementation Status](docs/IMPLEMENTATION_STATUS.md) - Current state and next steps
- [Unified Architecture](docs/UNIFIED_ARCHITECTURE.md) - Design rationale and patterns
- [V4 Source Mapping](docs/V4_SOURCE_MAPPING.md) - How V3 maps to V4

## Environment Variables

### API Server (`v4-server`)

| Variable | Default | Description |
|----------|---------|-------------|
| `V4_HOST` | 127.0.0.1 | Host to bind |
| `V4_PORT` | 8080 | Port to listen on |
| `V4_LOG_LEVEL` | info | Log level |
| `RUST_LOG` | - | Alternative log level |

### MCP Server (`v4-mcp-server`)

| Variable | Default | Description |
|----------|---------|-------------|
| `V4_MCP_HOST` | 127.0.0.1 | Host to bind |
| `V4_MCP_PORT` | 3000 | Port to listen on |
| `V4_LOG_LEVEL` | info | Log level |
| `RUST_LOG` | - | Alternative log level |

### A2A Worker (`a2a_worker`)

| Variable | Default | Description |
|----------|---------|-------------|
| `PROVIDER` | minimax | Provider: minimax, openrouter, deepseek, ollama, or custom |
| `MINIMAX_API_KEY` | - | MiniMax API key |
| `MINIMAX_HIGHSPEED` | false | Use MiniMax-M2.5-highspeed (2x output cost) |
| `OPENROUTER_API_KEY` | - | OpenRouter API key (for deepseek provider) |
| `OLLAMA_MODEL` | llama3.2 | Model name for Ollama |
| `BASE_URL` | - | Custom provider base URL |
| `API_KEY` | - | Custom provider API key |
| `MODEL` | - | Custom provider model |
| `HOST` | 127.0.0.1 | Host to bind |
| `PORT` | 3001 | Port to listen on |
| `RUST_LOG` | info | Log level |

## License

See workspace root for license information.
