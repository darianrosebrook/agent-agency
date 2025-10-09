# Agent Agency - Intelligent Multi-Agent Orchestration Platform

## Overview

**Agent Agency** is an intelligent, learning, and adaptive multi-agent orchestration platform built with TypeScript and following CAWS (Coding Agent Workflow System) v1.0 standards. The project uses a **mono-repo structure with iterations** to support progressive development from proof-of-concept to production-ready systems.

This mono-repo contains multiple iterations of the Agent Agency platform:

- **`iterations/poc/`**: Enhanced proof-of-concept with agentic RL, thinking budgets, multi-tenant memory, MCP integration, and advanced evaluation systems
- **`iterations/v1/`**: Empty - reserved for future production v1.0 implementation (when POC proves working concepts)

## Project Structure

```
agent-agency/
├── iterations/
│   ├── poc/              # Proof of Concept (CAWS Risk Tier 1)
│   └── main/             # Production v1.0 (CAWS Risk Tier 2)
├── docs/                 # Shared documentation
├── scripts/              # Shared build/deployment scripts
├── package.json          # Mono-repo management
└── tsconfig.json         # Base TypeScript configuration
```

## Active Iteration: Enhanced POC

The **enhanced POC iteration** (`iterations/poc/`) is the current active development focus, incorporating advanced features from recent AI research while remaining in proof-of-concept phase:

- **Agentic RL Foundations**: GRPO-style training concepts for tool use optimization
- **Thinking Budget Management**: Adaptive token allocation based on task complexity
- **Reward Hacking Prevention**: Minimal-diff checking and enhanced evaluation concepts
- **Multi-tenant Memory**: Advanced context offloading and federated learning
- **MCP Integration**: Full Model Context Protocol with autonomous reasoning
- **Advanced Reliability**: Preparing for production with mutation testing and performance budgets

The platform serves as an intelligent hub that:

- Registers and tracks different types of agents (workers, monitors, coordinators, orchestrators)
- Routes tasks to appropriate agents using intelligent, memory-aware algorithms
- Enables agents to learn from their experiences and share knowledge across projects
- Provides context offloading and hybrid RAG for efficient long-term memory management
- Supports multi-tenancy with secure data isolation and controlled cross-project learning
- Delivers autonomous reasoning and evaluation through local AI models
- Monitors system health with predictive analytics and trend analysis
- Supports continuous improvement through satisficing logic and federated learning

## Use Cases

This intelligent orchestration platform is designed for:

- **Multi-Project Agent Ecosystems**: Secure cross-project learning with tenant isolation
- **Autonomous Multi-Agent Systems**: Self-managing agent ecosystems with continuous learning
- **Intelligent Task Distribution**: Memory-aware routing that improves over time
- **Context Offloading Solutions**: Efficient long-term memory management for complex workflows
- **Federated Learning Platforms**: Privacy-preserving collective intelligence across organizations
- **Predictive System Monitoring**: AI-powered health monitoring with trend analysis
- **Adaptive Workflow Management**: Workflows that evolve based on experience and performance
- **Research & Development**: Advanced experimentation with learning agent coordination
- **Production AI Systems**: Enterprise-grade agent orchestration with autonomous reasoning

## Platform Capabilities

### **Core Orchestration**

- **Agent Lifecycle Management**: Register, monitor, and coordinate multiple agent types
- **Intelligent Task Orchestration**: Memory-aware task routing with predictive assignment
- **System Metrics & Analytics**: Real-time monitoring with trend analysis and forecasting
- **Type-Safe Architecture**: Full TypeScript implementation with comprehensive type definitions
- **Quality Gates**: Automated testing, linting, and CI/CD pipelines

### **Multi-Tenant Architecture**

- **Tenant Isolation**: Secure project-specific memory spaces with controlled sharing
- **Context Offloading**: Efficient LLM context management preventing "context rot"
- **Hybrid RAG**: Combined knowledge graph and vector embedding retrieval
- **Federated Learning**: Privacy-preserving cross-project intelligence sharing
- **Collective Intelligence**: Higher-order learning across the entire agent ecosystem

### **Advanced Features**

- **Agent Memory System**: Persistent knowledge graphs with vector embeddings and temporal reasoning
- **MCP Integration**: Model Context Protocol for autonomous AI reasoning and evaluation
- **Cross-Agent Learning**: Agents learn from each other's experiences and share knowledge
- **Predictive Analytics**: Forecast agent performance and task success probabilities
- **Autonomous Evaluation**: Built-in evaluation loops with satisficing logic

### **Agent Types Supported**

- **Workers**: Execute specific tasks (data processing, API calls, computations)
- **Monitors**: Continuously check system health and report metrics
- **Coordinators**: Facilitate communication between agents and manage workflows
- **Orchestrators**: High-level coordination and decision-making agents

Note: This POC provides the framework for these agent types, but actual agent implementations would be built on top of this orchestration layer.

## Architecture Overview

### **Mono-Repo Structure**

```mermaid
graph TB
    subgraph "Mono-Repo Root"
        MR[Mono-Repo<br/>Management]
        SD[Shared Docs<br/>& Scripts]
        CI[CI/CD<br/>Pipelines]
    end

    subgraph "Iterations"
        subgraph "Enhanced POC (iterations/poc/)"
            POC_SRC[Enhanced POC<br/>Source]
            POC_RL[Agentic RL<br/>Concepts]
            POC_THINK[Thinking<br/>Budgets]
            POC_EVAL[Enhanced<br/>Evaluation]
            POC_MEMORY[Multi-tenant<br/>Memory]
            POC_TESTS[POC Tests]
        end

        subgraph "Future v1.0 (iterations/v1/)"
            V1_SRC[Future v1.0<br/>Source]
            V1_PROD[Production<br/>Features]
        end
    end

    MR --> POC_SRC
    SD --> POC_DOCS
    CI --> POC_TESTS
```

### **Enhanced POC Architecture**

```mermaid
graph TB
    subgraph "Agentic RL System"
        RL[Agentic RL Trainer<br/>GRPO/PPO]
        CREDIT[Credit Assignment<br/>Tool Utility]
        REWARD[Model-based<br/>Rewarders]
    end

    subgraph "Thinking Budget Manager"
        TBM[Thinking Budget<br/>Manager]
        ADAPT[Adaptive Allocation<br/>Complexity-based]
        TRACK[Usage Tracking<br/>Efficiency Metrics]
    end

    subgraph "Enhanced Evaluation"
        EE[Enhanced Evaluator<br/>Minimal-diff]
        AST[AST Analysis<br/>Tree Edit Distance]
        JUDGE[LLM-as-Judge<br/>Faithfulness]
    end

    subgraph "Production Features"
        PERF[Performance<br/>Budgets]
        MUT[Mutation<br/>Testing 70%+]
        OBS[Observability<br/>Metrics]
    end

    RL --> CREDIT
    RL --> REWARD
    TBM --> ADAPT
    TBM --> TRACK
    EE --> AST
    EE --> JUDGE
    PERF --> MUT
    PERF --> OBS
```

### **Key Components by Iteration**

#### **Enhanced POC** (`iterations/poc/`)

##### **Agentic RL Concepts** (`iterations/poc/src/rl/`)

- **AgenticRLTrainer**: GRPO-style training framework for tool optimization
- **Credit Assignment**: Turn-level reward concepts for tool utility and thinking efficiency
- **Model-based Rewarders**: LLM judge architecture for faithfulness and minimal-diff evaluation

##### **Thinking Budget Manager** (`iterations/poc/src/thinking/`)

- **Adaptive Allocation**: Complexity-based token budget assignment framework
- **Performance Tracking**: Usage analytics and efficiency metrics
- **Historical Learning**: Budget optimization concepts from past task performance

##### **Enhanced Evaluation** (`iterations/poc/src/evaluation/`)

- **Minimal-diff Checking**: AST analysis and tree-edit distance validation
- **Reward Hacking Prevention**: Multi-criteria evaluation with safety gates
- **Tool Utility Scoring**: Credit assignment for tool call effectiveness

##### **Multi-Tenant Memory System** (`iterations/poc/src/memory/`)

- **Context Offloader**: Efficient LLM context management preventing "context rot"
- **Federated Learning Engine**: Privacy-preserving cross-project intelligence sharing
- **Tenant Isolator**: Secure data separation with controlled knowledge sharing

##### **MCP Integration** (`iterations/poc/src/mcp/`)

- **MCP Server**: Model Context Protocol server for autonomous AI reasoning
- **Tool Manager**: Comprehensive tool access for AI models with credit assignment
- **Evaluation Orchestrator**: Built-in evaluation loops with satisficing logic

##### **Data Layer** (`iterations/poc/src/data/`)

- **Multi-Level Cache**: Redis-based caching with intelligent promotion/demotion
- **Vector DAO**: PostgreSQL + pgvector for semantic similarity search
- **Performance Monitor**: Real-time metrics and trend analysis

##### **Advanced POC Features**

- **Mutation Testing**: Stryker-based testing framework with coverage requirements
- **Performance Budgets**: Latency and resource usage monitoring concepts
- **Enterprise Observability**: Advanced logging and metrics collection framework

### **Shared Components**

#### **Type System** (`iterations/*/src/types/`)

- Comprehensive type definitions across all iterations
- Consistent interfaces for agents, tasks, and system components
- Extensible type system for custom implementations

#### **Quality Assurance**

- **CAWS Compliance**: Risk Tier 1 standards for POC iterations
- **Multi-iteration Testing**: Parallel test execution across iterations
- **Shared CI/CD**: Unified pipelines with iteration-specific gates

## Quick Start

### **Prerequisites**

```bash
Node.js >= 18.0.0
npm >= 8.0.0
```

### **Installation**

```bash
# Clone and install
git clone <repository-url>
cd agent-agency
npm install
```

### **Working with Iterations**

```bash
# Install dependencies for all iterations
npm install

# Work with main iteration (production features)
npm run dev:main          # Start main iteration dev server
npm run build:main        # Build main iteration
npm run test:main         # Test main iteration

# Work with POC iteration (for comparison/reference)
npm run dev:poc           # Start POC dev server
npm run build:poc         # Build POC iteration
npm run test:poc          # Test POC iteration

# Run all tests across iterations
npm run test:all
```

### **Active Development: Main Iteration**

The main iteration includes advanced features like agentic RL and thinking budgets:

```bash
# Install and start main iteration
cd iterations/main
npm install
npm run dev

# Or from root directory
npm run dev:main
```

### **POC Iteration (Reference)**

The POC iteration contains the foundational implementation:

```bash
# Work with POC for reference
cd iterations/poc
npm install
npm run dev

# Or from root directory
npm run dev:poc
```

## Usage Examples

### **Enhanced Agent Registration with Memory**

```typescript
import { AgentOrchestrator } from "./src/services/AgentOrchestrator.js";

const orchestrator = new AgentOrchestrator();

// Register a worker agent with learning capabilities
const agentId = await orchestrator.registerAgent({
  name: "Data Processor",
  type: "worker",
  status: "idle",
  capabilities: ["process", "analyze", "learn"], // Enhanced capabilities
  metadata: {
    version: "1.0.0",
    learningEnabled: true,
    memoryProfile: "adaptive",
  },
});

console.log(`Agent registered with ID: ${agentId}`);
```

### **Intelligent Task Submission with Context**

```typescript
// Submit a task with context for memory-aware routing
const taskId = await orchestrator.submitTaskWithContext({
  agentId, // Which agent should handle this
  type: "process", // Type of task (matches agent capabilities)
  payload: {
    data: "sample input",
    options: { priority: "high" },
  },
  context: {
    query: "process customer data with privacy filtering",
    requiredCapabilities: ["data-processing", "privacy"],
    expectedOutcome: "filtered-dataset",
  },
});

console.log(`Task submitted with ID: ${taskId}`);

// Predict task success probability
const prediction = await orchestrator.predictTaskSuccess(taskId, agentId);
console.log(`Success probability: ${prediction.probability * 100}%`);
```

### **Advanced System Monitoring and Analytics**

```typescript
// Get comprehensive system metrics with trends
const metrics = await orchestrator.getSystemMetrics();
console.log(`Active Agents: ${metrics.activeAgents}`);
console.log(`Total Tasks: ${metrics.totalTasks}`);
console.log(`Completed Tasks: ${metrics.completedTasks}`);
console.log(`Failed Tasks: ${metrics.failedTasks}`);

// Get agent memory insights
const memoryInsights = await orchestrator.getAgentMemoryInsights(agentId);
console.log(`Agent Learning Rate: ${memoryInsights.learningRate}`);
console.log(`Capability Evolution: ${memoryInsights.capabilityEvolution}`);
console.log(`Cross-Agent Learning: ${memoryInsights.crossAgentLearning}`);

// Find similar agents for collaboration
const similarAgents = await orchestrator.findSimilarAgents({
  capabilities: ["data-processing"],
  minSimilarity: 0.8,
  excludeSelf: true,
});
console.log(`Similar agents found: ${similarAgents.length}`);
```

### **Memory-Aware Agent Learning**

```typescript
// Learn from task outcome
await orchestrator.learnFromTaskOutcome(taskId, {
  status: "success",
  performanceScore: 0.92,
  learnedCapabilities: ["privacy-filtering", "data-validation"],
  lessonsLearned: ["Improved data sanitization process"],
  executionTime: 1500,
  capabilitiesUsed: ["process", "analyze", "validate"],
});

// Get agent task history for analysis
const taskHistory = await orchestrator.getTaskHistory(agentId);
console.log(`Agent has completed ${taskHistory.length} tasks`);
console.log(
  `Average performance: ${
    taskHistory.reduce((sum, t) => sum + t.performanceScore, 0) /
    taskHistory.length
  }`
);
```

### **Multi-Tenant Context Offloading**

```typescript
// Initialize multi-tenant memory system
const tenantId = "project-alpha";
const multiTenantMemory = new MultiTenantMemoryManager(config);

// Offload complex context to prevent LLM context rot
const contextRef = await multiTenantMemory.offloadContext(tenantId, {
  task: "complex-data-analysis",
  requirements: ["privacy-compliance", "performance-optimization"],
  historicalPatterns: ["similar-analysis-2024-01", "data-validation-2024-02"],
  constraints: ["gdpr-compliant", "sub-500ms-response"],
});

// Later retrieve and reconstruct context for new similar task
const reconstructedContext = await multiTenantMemory.retrieveContext(
  contextRef,
  tenantId
);

// Use federated learning insights across projects
const federatedInsights = await multiTenantMemory.getFederatedInsights(
  tenantId,
  currentTaskContext
);
console.log(`Found ${federatedInsights.length} cross-project insights`);
```

## Configuration

### **Enhanced Agent Orchestrator Config**

```typescript
const config: EnhancedAgentOrchestratorConfig = {
  // Core orchestration settings
  maxConcurrentTasks: 10,
  taskTimeoutMs: 30000,
  retryAttempts: 3,
  healthCheckIntervalMs: 5000,

  // Memory system configuration
  memory: {
    enableMemorySystem: true,
    embeddingModel: "embeddinggemma",
    embeddingDimension: 768,
    similarityThreshold: 0.7,
    maxMemoryRetrieval: 20,
  },

  // MCP integration settings
  mcp: {
    enableMCP: true,
    localModel: "gemma:3n",
    evaluationEnabled: true,
    satisficingThreshold: 0.8,
  },

  // Database configuration
  database: {
    url: "postgresql://user:pass@localhost:5432/agent_agency",
    poolSize: 20,
    enableVectorSearch: true,
  },

  // Caching configuration
  cache: {
    redis: {
      url: "redis://localhost:6379",
      ttl: 3600,
    },
  },
};
```

## API Contract

The system exposes a comprehensive REST API defined in OpenAPI 3.0 format (`apps/contracts/api.yaml`):

- **Agent Management**: Register, list, and monitor agents
- **Task Management**: Submit, track, and manage tasks
- **System Metrics**: Real-time system health and performance data

## CAWS Compliance

This project follows CAWS v1.0 standards for engineering-grade agent development, which provides:

- **Risk Tier 2**: Standard rigor with comprehensive testing requirements (80% branch coverage, 50% mutation testing)
- **Working Spec**: Formal requirements document in `.caws/working-spec.yaml` with defined acceptance criteria
- **Quality Gates**: Automated CI/CD pipeline that enforces testing, linting, type checking, and security scans
- **Provenance Tracking**: Complete audit trail of all changes, tool usage, and approvals in `.agent/provenance.json`
- **Contract-First Development**: OpenAPI specification defined before implementation to ensure API consistency

This compliance ensures the codebase maintains high quality standards and provides transparency about how changes are made and validated.

## Testing Strategy

### **Unit Tests** (`tests/unit/`)

- Agent orchestration logic
- Task management workflows
- System metrics calculation
- Error handling and edge cases

### **Coverage Requirements**

- **Branches**: ≥70%
- **Functions**: ≥70%
- **Lines**: ≥70%
- **Statements**: ≥70%

## Security & Reliability

- **Input Validation**: Type-safe interfaces prevent invalid data
- **Error Handling**: Comprehensive error boundaries and logging
- **Health Monitoring**: Built-in agent health checks
- **Idempotent Operations**: Safe retry mechanisms
- **Audit Logging**: Complete operation traceability

## Performance Characteristics

- **Concurrent Tasks**: Configurable concurrency limits
- **Task Timeouts**: Configurable execution timeouts
- **Memory Efficient**: In-memory data structures (POC scope)
- **Scalable Architecture**: Designed for distributed deployment

## Current Implementation Status

### **Implemented Features** ✅

- ✅ **Core Agent Orchestration**: Complete agent registration, task routing, and system metrics
- ✅ **MCP Integration**: Full Model Context Protocol server with tools and resource management
- ✅ **Multi-Tenant Memory System**: Context offloading, tenant isolation, and federated learning framework
- ✅ **Data Layer**: PostgreSQL + pgvector support with multi-level Redis caching
- ✅ **Type-Safe Architecture**: Comprehensive TypeScript implementation with proper error handling
- ✅ **Quality Gates**: Automated testing (98 passing tests), linting, and CI/CD pipelines
- ✅ **CAWS Compliance**: Engineering-grade development practices with comprehensive testing

### **Architecture Components** 🏗️

- ✅ **Agent Orchestrator**: Memory-aware task routing and capability evolution
- ✅ **Memory Management**: Multi-tenant context offloading and knowledge graph foundations
- ✅ **Database Operations**: Full CRUD operations with vector similarity search framework
- ✅ **Caching Layer**: Multi-level Redis caching with intelligent promotion/demotion
- ✅ **Testing Infrastructure**: Comprehensive unit and integration test suites
- ✅ **MCP Server**: Autonomous AI reasoning with evaluation loops and satisficing logic

### **In Development** 🚧

- 🚧 **Vector Embeddings**: Semantic similarity search and context-aware retrieval algorithms
- 🚧 **Knowledge Graph Engine**: Entity extraction, relationship building, and graph traversal
- 🚧 **Intelligent Task Routing**: Memory-aware predictive assignment and performance forecasting
- 🚧 **Cross-Agent Learning**: Experience sharing and capability evolution
- 🚧 **Federated Learning**: Privacy-preserving cross-project intelligence algorithms

### **Planned Features** 📋

- 📋 **Production Database Schema**: Complete PostgreSQL schema with optimized indexes
- 📋 **Advanced Scheduling**: Priority queues and distributed task distribution
- 📋 **Security Framework**: Authentication, authorization, and comprehensive audit logging
- 📋 **Real-time Monitoring**: Advanced dashboards and predictive analytics
- 📋 **Distributed Architecture**: Multi-node deployment with horizontal scaling

## Development Roadmap

### **Phase 1: Core Memory Infrastructure (Weeks 1-4)**

- [ ] Database setup with PostgreSQL and pgvector
- [ ] Core memory tables and vector similarity indexes
- [ ] Basic memory services and embedding integration
- [ ] Experience extraction and knowledge graph foundation

### **Phase 2: Knowledge Graph Integration (Weeks 5-8)**

- [ ] Entity extraction and relationship building
- [ ] Semantic search and context-aware retrieval
- [ ] Agent matching and similarity search
- [ ] Conversation memory with embeddings

### **Phase 3: Advanced Reasoning (Weeks 9-12)**

- [ ] Multi-hop reasoning and intelligent task routing
- [ ] Temporal analysis and performance prediction
- [ ] Causality detection and trend forecasting
- [ ] Cross-agent learning implementation

### **Phase 4: Multi-Tenant Architecture (Weeks 13-16)**

- [ ] Context offloading mechanisms and quarantine
- [ ] Multi-tenant data isolation and access control
- [ ] Federated learning with privacy preservation
- [ ] Hybrid RAG combining graph and vector search

### **Phase 5: Production Integration (Weeks 17-20)**

- [ ] Multi-tenant system integration and optimization
- [ ] Security framework with tenant authentication
- [ ] Distributed architecture with tenant scaling
- [ ] Advanced monitoring for multi-tenant operations

### **Future Enhancements**

- [ ] Federated learning across multiple instances
- [ ] Multi-modal memory support (images, audio, video)
- [ ] External knowledge base integrations
- [ ] Real-time learning and adaptation
- [ ] Enterprise-grade deployment automation

## Contributing

This project follows CAWS v1.0 development practices:

1. **Working Spec**: Define requirements in `.caws/working-spec.yaml`
2. **Test First**: Write tests before implementation
3. **Quality Gates**: All changes must pass CI/CD validation
4. **Documentation**: Update contracts and specifications

## License

MIT License - see LICENSE file for details.

## Author

**@darianrosebrook**

## Related Projects

- [CAWS Framework](https://github.com/paths-design/caws) - The underlying workflow system
- [Agent Protocol](https://agentprotocol.ai) - Agent communication standards
- [AutoGen](https://microsoft.github.io/autogen/) - Multi-agent conversation framework

## Documentation

- **[Multi-Tenant Memory System](docs/memory-system/README.md)**: Multi-tenant architecture and context offloading
- **[Multi-Tenancy & Context Offloading](docs/memory-system/multi-tenancy.md)**: Comprehensive multi-tenant implementation guide
- **[Agent Memory System](docs/memory-system/README.md)**: Core memory and learning capabilities
- **[MCP Integration](docs/MCP/README.md)**: Model Context Protocol for autonomous AI reasoning
- **[Technical Architecture](docs/memory-system/technical-architecture.md)**: Deep technical implementation details
- **[Implementation Roadmap](docs/memory-system/implementation-roadmap.md)**: Detailed development timeline and milestones

## Technology Stack

- **Backend**: TypeScript, Node.js, Fastify
- **Database**: PostgreSQL with pgvector extension
- **Caching**: Redis for high-performance memory operations
- **AI/ML**: Ollama with embedding models (embeddinggemma)
- **Protocols**: MCP (Model Context Protocol) for AI model integration
- **Quality**: CAWS v1.0 compliance with comprehensive testing

---

**Agent Agency is currently in enhanced proof-of-concept phase with a consolidated iteration containing advanced concepts from recent AI research. The POC demonstrates comprehensive functionality including multi-tenant memory systems, MCP integration, agentic RL frameworks, thinking budget management, and enhanced evaluation systems. When the POC proves working concepts through testing and validation, the production v1.0 implementation will be developed in the reserved iterations/v1/ directory.**
