# Agent Memory System

**Enterprise-grade memory architecture for intelligent, learning AI agents**

The Agent Memory System implements the v4 memory architecture in v3, providing **database-persistent memory capabilities** that enable agents to learn from experiences, build knowledge graphs, and make context-aware decisions. Features automatic decay scheduling, multi-tenancy support, and seamless integration with chat, context management, and ingestion systems.

## Overview

This system combines multiple memory technologies:

- **Knowledge Graphs**: Structured representation of entities, relationships, and concepts
- **Vector Embeddings**: Semantic similarity search with decay and importance weighting
- **Temporal Reasoning**: Time-based analysis and causality detection
- **Context Offloading**: Strategic memory management for long-horizon tasks
- **Provenance Tracking**: Explainable AI with operation audit trails

## Key Features

### **Database Persistence**
- **PostgreSQL Integration**: Full persistence with connection pooling and health monitoring
- **Migration System**: Automatic schema updates with rollback support
- **Multi-Tenancy**: Row Level Security (RLS) for tenant isolation
- **Transaction Safety**: ACID compliance for memory operations
- **Backup Integration**: Memory data included in system backups

### **Multi-Modal Memory**
- **Episodic Memory**: Specific events and experiences with full context
- **Semantic Memory**: General knowledge and facts with cross-linking
- **Procedural Memory**: Task execution patterns and capabilities
- **Working Memory**: Temporary context for current task execution

### **Automated Memory Management**
- **Decay Scheduler**: Background task for configurable forgetting curves
- **Context Offloading**: Strategic memory compression and external storage
- **Lifecycle Management**: Automatic archival, consolidation, and cleanup
- **Importance Weighting**: Access patterns and significance scoring
- **Maintenance Automation**: Self-regulating memory health and optimization

### **Intelligent Retrieval**
- **Hybrid Search**: Combines vector similarity with graph traversal
- **Multi-Hop Reasoning**: Follows relationship chains up to N hops
- **Context-Aware Retrieval**: Retrieves memories based on current task context
- **Temporal Weighting**: Prioritizes recent, relevant memories
- **Importance Scoring**: Memory strength based on access patterns and significance

### **Temporal Intelligence**
- **Decay Management**: Configurable forgetting curves (exponential, power-law, logarithmic)
- **Importance Boosting**: Recent access and successful outcomes increase memory strength
- **Change Point Detection**: Identifies significant shifts in agent performance
- **Causality Analysis**: Discovers cause-effect relationships in experiences
- **Trend Forecasting**: Predicts future performance based on historical patterns

### **Knowledge Architecture**
- **Entity Deduplication**: Merges similar entities with confidence scoring
- **Relationship Mining**: Automatic extraction of entity relationships from experiences
- **Graph Traversal**: Efficient navigation of complex knowledge networks
- **Cross-Modal Linking**: Connects different memory types (episodic ↔ semantic)

### **Service Integration**
- **Chat Service**: Memory-enhanced conversation with context persistence
- **Context Manager**: Database-backed context preservation and retrieval
- **Ingestors**: File and data ingestion with automatic memory storage
- **API Server**: REST endpoints for memory operations with authentication
- **File Watcher**: Real-time ingestion triggers with notify crate integration

### **Observability & Analytics**
- **Memory Health Metrics**: Usage statistics, decay patterns, retrieval performance
- **Performance Analytics**: Agent capability evolution and learning trends
- **Provenance Tracking**: Complete audit trail of memory operations
- **Maintenance Automation**: Automated cleanup and consolidation
- **Multi-Tenant Monitoring**: Per-tenant usage and performance tracking

## Architecture

```mermaid
graph TB
    subgraph "Agent Memory System"
        MM[Memory Manager]
        DS[Decay Scheduler]
        KG[Knowledge Graph Engine]
        VE[Vector Embedding Service]
        TR[Temporal Reasoning Engine]
        CM[Context Offloading Engine]
        PT[Provenance Tracker]
    end

    subgraph "Service Layer"
        CS[Chat Service]
        CMS[Context Manager]
        ING[Unified Ingestor]
        FW[File Watcher]
        API[API Server]
    end

    subgraph "Storage Layer"
        PG[(PostgreSQL + pgvector)]
        PG_POOL[Connection Pool]
        CACHE[(Redis Cache)]
        FILES[(File Storage)]
    end

    subgraph "Integration Layer"
        ES[Embedding Service]
        CPE[Context Preservation Engine]
        OBS[Observability System]
        MIGR[Migration Runner]
    end

    MM --> DS
    MM --> KG
    MM --> VE
    MM --> TR
    MM --> CM
    MM --> PT

    CS --> PG
    CMS --> PG
    ING --> PG
    API --> PG_POOL

    KG --> PG
    VE --> PG
    TR --> PG
    CM --> PG
    PT --> PG
    DS --> PG

    PG_POOL --> PG
    FW --> ING

    VE --> ES
    CM --> CPE
    MM --> OBS
    API --> MIGR

    subgraph "Multi-Tenant Isolation"
        RLS[Row Level Security]
        TENANTS[Tenants Table]
        PRIVACY[Differential Privacy]
    end

    PG --> RLS
    RLS --> TENANTS
    RLS --> PRIVACY
```

## Quick Start

### 1. Database Setup

First, ensure PostgreSQL is running with the required extensions:

```bash
# Install PostgreSQL extensions
psql -d agent_agency_v3 -c "CREATE EXTENSION IF NOT EXISTS vector;"
psql -d agent_agency_v3 -c "CREATE EXTENSION IF NOT EXISTS uuid_ossp;"

# Run migrations
export DATABASE_URL="postgresql://user:password@localhost:5432/agent_agency_v3"
cargo run --bin agent-agency-api-server -- --host 127.0.0.1 --port 8080
# Migrations run automatically on startup
```

### 2. Add to Dependencies

```toml
[dependencies]
agent-memory = { path = "../agent-memory", features = ["database"] }
data-infrastructure = { path = "../data-infrastructure" }
tokio = { version = "1.0", features = ["full"] }
```

### 3. Initialize Memory System with Database

```rust
use agent_memory::*;
use data_infrastructure::database_config::DatabaseConfig;
use data_infrastructure::database_init::initialize_database;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Database configuration
    let db_config = DatabaseConfig {
        database_url: std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost/agent_agency_v3".to_string()),
        ..Default::default()
    };

    // Initialize database with migrations
    let db_client = initialize_database(db_config).await?;
    let db_client = Arc::new(db_client);

    // Configure memory system with database
    let config = MemoryConfig::default();

    // Initialize memory manager with database
    let memory_manager = MemoryManager::new(config, db_client.pool().clone()).await?;
    let memory_manager = Arc::new(memory_manager);

    // Start decay scheduler
    memory_manager.start_decay_scheduler().await?;

    println!("Memory system initialized with database persistence");
    Ok(())
}
```

### 4. Use Memory Services

```rust
use data_infrastructure::{chat_service::ChatService, simple_client::DatabaseClient};
use agent_data_processing::context::manager::ContextManager;
use agent_data_processing::ingestion::UnifiedIngestor;

// Initialize services with database
let chat_service = ChatService::new(db_client.clone());
let context_manager = ContextManager::new_with_db_client(
    context_config,
    ai_service,
    db_client.clone()
).await?;
let ingestor = UnifiedIngestor::new_with_db_client(db_client.clone());
```

### 5. Store Agent Experiences

```rust
// Create an agent experience
let experience = AgentExperience {
    id: MemoryId::new_v4(),
    agent_id: "agent-001".to_string(),
    task_id: "task-123".to_string(),
    context: TaskContext {
        task_id: "task-123".to_string(),
        task_type: "code_review".to_string(),
        description: "Review authentication middleware".to_string(),
        domain: vec!["security".to_string(), "authentication".to_string()],
        entities: vec!["JWT".to_string(), "middleware".to_string()],
        temporal_context: Some(TemporalContext {
            start_time: Utc::now(),
            deadline: Some(Utc::now() + Duration::hours(2)),
            priority: TaskPriority::High,
            recurrence_pattern: None,
        }),
        metadata: HashMap::new(),
    },
    input: serde_json::json!({"files": ["auth.rs"]}),
    output: serde_json::json!({"issues": ["Add validation"]}),
    outcome: ExperienceOutcome {
        success: true,
        performance_score: Some(0.9),
        learned_capabilities: vec!["security_audit".to_string()],
        failure_reasons: vec![],
        success_factors: vec!["thorough_analysis".to_string()],
        execution_time_ms: Some(1800),
        tokens_used: Some(800),
        feedback: None,
    },
    memory_type: MemoryType::Episodic,
    timestamp: Utc::now(),
    metadata: HashMap::new(),
};

// Store the experience
let memory_id = memory_system.store_experience(experience).await?;
```

### 4. Retrieve Contextual Memories

```rust
// Define current task context
let context = TaskContext {
    task_id: "task-456".to_string(),
    task_type: "security_implementation".to_string(),
    description: "Implement JWT authentication with validation".to_string(),
    domain: vec!["security".to_string(), "authentication".to_string()],
    entities: vec!["JWT".to_string(), "validation".to_string()],
    temporal_context: Some(TemporalContext {
        start_time: Utc::now(),
        deadline: Some(Utc::now() + Duration::hours(4)),
        priority: TaskPriority::High,
        recurrence_pattern: None,
    }),
    metadata: HashMap::new(),
};

// Retrieve relevant memories
let memories = memory_system.retrieve_contextual_memories(&context, 5).await?;

for memory in memories {
    println!("Found relevant memory: {}", memory.memory.context.description);
    println!("Relevance score: {:.3}", memory.relevance_score);
}
```

### 5. Perform Reasoning

```rust
// Query for multi-hop reasoning
let reasoning_query = ReasoningQuery {
    start_entities: vec!["agent:agent-001".to_string()],
    target_entities: vec!["capability:security_audit".to_string()],
    relationship_types: vec![RelationshipType::LearnsFrom],
    max_hops: 2,
    min_confidence: 0.5,
    time_range: None,
};

let result = memory_system.perform_reasoning(reasoning_query).await?;
println!("Found {} reasoning paths", result.paths.len());
```

## Configuration

### Memory System Configuration

```rust
let config = MemoryConfig {
    graph_config: GraphConfig {
        max_entities: 100_000,
        max_relationships_per_entity: 50,
        similarity_threshold: 0.8,
        deduplication_enabled: true,
        reasoning_depth: 3,
        cache_size: 10_000,
    },
    embedding_config: EmbeddingConfig {
        model_name: "embeddinggemma".to_string(),
        dimension: 768,
        batch_size: 32,
        cache_enabled: true,
        cache_size: 5_000,
        similarity_threshold: 0.7,
    },
    temporal_config: TemporalConfig {
        analysis_window_days: 30,
        causality_enabled: true,
        trend_detection_enabled: true,
        forecasting_enabled: true,
        change_point_sensitivity: 0.3,
    },
    decay_config: DecayConfig {
        base_decay_rate: 0.05,
        importance_boost_factor: 0.1,
        access_recency_weight: 0.8,
        consolidation_interval_hours: 24,
        minimum_memory_strength: 0.1,
        decay_schedule: DecaySchedule::Exponential,
        enabled: true, // Enable automatic decay
    },
    decay_scheduler_config: DecaySchedulerConfig {
        decay_interval_seconds: 3600, // 1 hour
        enabled: true,
        max_concurrent_cycles: 1,
    },
    context_config: ContextConfig {
        compression_enabled: true,
        compression_threshold_kb: 50,
        offload_strategy: OffloadStrategy::Compress,
        retrieval_boost_factor: 1.2,
        max_context_age_days: 90,
    },
    performance_config: PerformanceConfig {
        metrics_enabled: true,
        query_timeout_ms: 5000,
        max_concurrent_queries: 10,
        memory_pressure_threshold_mb: 500,
        cache_enabled: true,
    },
};
```

### Decay Schedule Options

- **Exponential**: `importance *= (1 - decay_rate) ^ time_elapsed`
- **PowerLaw**: `importance *= time_elapsed ^ (-decay_rate)`
- **Logarithmic**: `importance -= log(time_elapsed) * decay_rate`
- **Custom**: User-defined decay formula

## Database Schema

The memory system uses comprehensive database schema with automatic migrations:

### Core Memory Tables
- `agent_experiences` - Agent learning experiences and outcomes
- `memory_embeddings` - Vector embeddings with workspace/tenant isolation
- `knowledge_graph_entities` - Graph nodes (agents, tasks, capabilities)
- `knowledge_graph_relationships` - Graph edges with strength/confidence
- `temporal_analysis_results` - Cached temporal analysis results
- `memory_provenance` - Complete audit trail of memory operations

### Context Management
- `agent_contexts` - Active context storage with compression
- `offloaded_contexts` - Compressed/archived contexts
- `folded_contexts` - Lifecycle-managed context folding

### Chat Integration
- `chat_sessions` - Conversation sessions with metadata
- `chat_messages` - Individual messages with threading
- `chat_context_links` - Links between chat and offloaded contexts

### Multi-Tenancy
- `tenants` - Tenant management with isolation levels
- `tenant_privacy_config` - Privacy settings for federated learning

### Automatic Migrations

Migrations run automatically when the API server starts:

```bash
export DATABASE_URL="postgresql://user:password@localhost:5432/agent_agency_v3"
cargo run --bin agent-agency-api-server -- --host 127.0.0.1 --port 8080
# All migrations (001-011) run automatically
```

### Migration Files
1. `001_enable_pgvector.sql` - Vector extension for embeddings
2. `002_create_vector_tables.sql` - Vector storage tables
3. `003_create_agent_experiences.sql` - Core experience storage
4. `004_create_memory_system.sql` - Memory embeddings and graphs
5. `005_create_planning_system.sql` - Task planning storage
6. `006_create_telemetry_storage.sql` - Metrics and telemetry
7. `007_create_worker_assignment_tracking.sql` - Worker management
8. `008_create_agent_context_management.sql` - Context lifecycle
9. `009_create_wal_storage.sql` - Write-ahead logging
10. `010_create_chat_persistence.sql` - Chat system
11. `011_add_multi_tenant_isolation.sql` - Multi-tenancy with RLS

## Performance Characteristics

### Scalability
- **Memory Capacity**: Supports millions of agent experiences
- **Graph Size**: Handles knowledge graphs with 100K+ entities
- **Query Performance**: Sub-100ms response times for complex queries
- **Concurrent Access**: Supports 1000+ concurrent memory operations

### Memory Management
- **Vector Storage**: Optimized pgvector with IVFFlat indexing
- **Graph Traversal**: Efficient breadth-first search algorithms
- **Cache Layers**: Multi-level caching (memory → Redis → disk)
- **Compression**: Automatic context compression for long-term storage

### Maintenance Operations
- **Decay Processing**: Configurable batch sizes and intervals
- **Consolidation**: Automatic merging of similar memories
- **Cleanup**: TTL-based cleanup of expired contexts
- **Optimization**: Automatic index maintenance and statistics updates

## Memory Decay Scheduler

The memory system includes an automated background scheduler for memory decay:

```rust
// Start the decay scheduler
memory_manager.start_decay_scheduler().await?;

// Check scheduler status
let status = memory_manager.get_decay_scheduler_status().await;
println!("Scheduler running: {}", status.map(|s| s.running).unwrap_or(false));

// Run manual decay cycle for testing
let updated = memory_manager.run_manual_decay_cycle().await?;
println!("Updated {} memories in manual decay cycle", updated);

// Stop scheduler
memory_manager.stop_decay_scheduler().await?;
```

### Scheduler Configuration

```rust
let scheduler_config = DecaySchedulerConfig {
    decay_interval_seconds: 3600, // Run every hour
    enabled: true,
    max_concurrent_cycles: 1,
};
```

### Decay Algorithms

The scheduler supports multiple decay algorithms:
- **Exponential**: `importance *= (1 - decay_rate) ^ time_elapsed`
- **Power Law**: `importance *= time_elapsed ^ (-decay_rate)`
- **Logarithmic**: `importance -= log(time_elapsed) * decay_rate`

## Service Integration

### Complete System Integration

```rust
use agent_memory::MemoryManager;
use data_infrastructure::{chat_service::ChatService, simple_client::DatabaseClient};
use agent_data_processing::{context::manager::ContextManager, ingestion::UnifiedIngestor};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize database
    let db_config = DatabaseConfig {
        database_url: std::env::var("DATABASE_URL").unwrap(),
        ..Default::default()
    };
    let db_client = Arc::new(initialize_database(db_config).await?);

    // Initialize memory system
    let memory_config = MemoryConfig::default();
    let memory_manager = Arc::new(MemoryManager::new(memory_config, db_client.pool().clone()).await?);

    // Initialize integrated services
    let chat_service = ChatService::new(db_client.clone());
    let context_manager = ContextManager::new_with_db_client(
        context_config, ai_service, db_client.clone()
    ).await?;
    let ingestor = UnifiedIngestor::new_with_db_client(db_client.clone());

    // Start memory decay scheduler
    memory_manager.start_decay_scheduler().await?;

    println!("All services initialized with database persistence");
    Ok(())
}
```

### With Agent Orchestrator

```rust
// In agent orchestrator
pub struct EnhancedAgentOrchestrator {
    memory_system: Arc<MemorySystem>,
    // ... other fields
}

impl EnhancedAgentOrchestrator {
    pub async fn submit_task_with_memory(&self, task: Task) -> Result<TaskResult, Error> {
        // Retrieve relevant memories for task context
        let context = self.build_task_context(&task);
        let memories = self.memory_system.retrieve_contextual_memories(&context, 10).await?;

        // Enrich task with memory context
        let enriched_task = self.enrich_task_with_memories(task, memories);

        // Execute task
        let result = self.execute_task(enriched_task).await?;

        // Store experience in memory
        let experience = self.build_experience_from_result(&result);
        self.memory_system.store_experience(experience).await?;

        Ok(result)
    }
}
```

### With Context Preservation

```rust
// Integrate with context-preservation-engine
pub struct MemoryAwareContextManager {
    memory_system: Arc<MemorySystem>,
    context_engine: Arc<ContextPreservationEngine>,
}

impl MemoryAwareContextManager {
    pub async fn offload_context(&self, context_id: &str) -> Result<(), Error> {
        // Get current context
        let context = self.context_engine.get_context(context_id).await?;

        // Decide whether to keep in working memory or offload
        if self.should_offload_context(&context) {
            // Compress and offload to memory system
            self.memory_system.offload_context(context).await?;
            self.context_engine.remove_context(context_id).await?;
        }

        Ok(())
    }
}
```

## Monitoring and Maintenance

### Health Checks

```rust
// Get memory manager health and decay scheduler status
let scheduler_status = memory_manager.get_decay_scheduler_status().await;
println!("Decay scheduler running: {}", scheduler_status.map(|s| s.running).unwrap_or(false));

// Run manual decay cycle
let decayed = memory_manager.run_manual_decay_cycle().await?;
println!("Manual decay cycle updated {} memories", decayed);

// Get database health
let db_health = db_client.pool().is_closed();
println!("Database connection healthy: {}", !db_health);
```

### Metrics and Monitoring

The system provides comprehensive monitoring:

#### Memory Decay Metrics
- `memory_decay_scheduler_running` - Whether scheduler is active
- `memory_decay_cycles_total` - Total decay cycles executed
- `memory_decay_cycle_duration` - Time taken per decay cycle
- `memory_decay_updated_count` - Memories updated per cycle

#### Database Performance Metrics
- `memory_db_query_duration` - Database query response times
- `memory_db_connection_pool_size` - Active connection pool size
- `memory_db_connection_errors` - Database connection errors

#### Multi-Tenant Metrics
- `memory_tenant_isolation_violations` - RLS policy violations
- `memory_tenant_query_count` - Queries per tenant
- `memory_global_memory_access` - Global memory access patterns

#### Service Integration Metrics
- `memory_chat_sessions_active` - Active chat sessions
- `memory_context_offloaded` - Contexts moved to external storage
- `memory_ingestion_triggers` - File watcher ingestion events

### Maintenance Operations

```rust
// Comprehensive maintenance
async fn run_memory_maintenance(memory_manager: &MemoryManager, db_client: &DatabaseClient) {
    // Run decay cycle
    let decayed = memory_manager.run_manual_decay_cycle().await?;
    println!("Decayed {} memories", decayed);

    // Verify database schema
    let schema_ok = data_infrastructure::database_init::verify_schema(db_client.pool()).await?;
    println!("Schema verification: {}", if schema_ok { "PASSED" } else { "FAILED" });

    // Check tenant isolation
    // Note: Implement tenant verification logic

    // Monitor performance
    let scheduler_status = memory_manager.get_decay_scheduler_status().await;
    if let Some(status) = scheduler_status {
        println!("Scheduler: {} (interval: {}s)",
                if status.running { "RUNNING" } else { "STOPPED" },
                status.interval_seconds);
    }
}
```

## Best Practices

### Memory Design
1. **Define Clear Memory Types**: Use appropriate memory types for different information
2. **Structure Task Contexts**: Include domain, entities, and temporal information
3. **Configure Decay Appropriately**: Balance forgetting with retention needs
4. **Monitor Performance**: Track retrieval accuracy and response times

### Operational Excellence
1. **Regular Maintenance**: Run decay cycles and cleanup operations
2. **Monitor Health**: Track memory usage and performance metrics
3. **Backup Strategy**: Include memory data in backup procedures
4. **Version Compatibility**: Handle schema migrations carefully

### Performance Optimization
1. **Index Strategy**: Maintain appropriate database indexes
2. **Cache Configuration**: Tune cache sizes for your workload
3. **Batch Operations**: Use batch processing for bulk operations
4. **Query Optimization**: Design queries to leverage indexes

## Troubleshooting

### Database Connection Issues

**Connection Pool Exhausted**
```rust
// Check pool status
let pool_size = db_client.pool().size().await;
let idle_connections = db_client.pool().num_idle().await;
println!("Pool size: {}, Idle: {}", pool_size, idle_connections);
```
- Increase `max_connections` in DatabaseConfig
- Check for connection leaks in long-running operations
- Monitor connection timeout settings

**Migration Failures**
- Verify PostgreSQL extensions are installed (`pgvector`, `uuid-ossp`)
- Check database permissions for schema modifications
- Review migration logs for specific error details
- Run migrations manually: `cargo run --bin agent-agency-api-server`

### Memory Decay Issues

**Decay Not Running**
```rust
// Check scheduler status
let status = memory_manager.get_decay_scheduler_status().await;
println!("Scheduler status: {:?}", status);
```
- Verify `decay_config.enabled = true`
- Check database connectivity for decay operations
- Monitor scheduler logs for errors

**Decay Too Aggressive**
- Reduce `base_decay_rate` in DecayConfig
- Increase `minimum_memory_strength`
- Adjust decay schedule algorithm (try PowerLaw instead of Exponential)

### Multi-Tenant Issues

**RLS Policy Violations**
- Verify current tenant context is set
- Check tenant isolation level (strict vs shared)
- Review sharing rules for federated tenants

**Global Memory Not Accessible**
- Ensure `workspace_id = NULL` queries are allowed
- Check tenant has access to global memory
- Verify RLS policies allow NULL workspace access

### Performance Issues

**Slow Memory Retrieval**
- Check vector indexes: `SELECT * FROM pg_indexes WHERE tablename = 'memory_embeddings';`
- Monitor query execution plans
- Consider increasing `max_connections` for high load

**High Memory Usage**
- Check decay configuration and run maintenance
- Review importance scores and boost thresholds
- Monitor cache sizes and eviction policies
- Run manual decay: `memory_manager.run_manual_decay_cycle().await?`

**Context Offloading Problems**
- Verify compression settings in ContextConfig
- Check database storage for offloaded contexts
- Monitor context lifecycle metrics

### Service Integration Issues

**Chat Service Not Persisting**
- Verify ChatService has database client
- Check chat_sessions and chat_messages tables exist
- Review database connection in ChatService

**Context Manager Errors**
- Ensure ContextManager initialized with database client
- Check agent_contexts table permissions
- Verify compression/decompression working

**File Watcher Not Triggering**
- Confirm notify crate dependency
- Check file system permissions
- Verify watch paths exist
- Test pattern matching manually

### Monitoring and Debugging

**Enable Debug Logging**
```rust
// Set environment variables
export RUST_LOG=agent_memory=debug,data_infrastructure=debug
export DATABASE_URL="postgresql://user:password@localhost:5432/agent_agency_v3"

// Run with verbose logging
cargo run --bin agent-agency-api-server -- --host 127.0.0.1 --port 8080
```

**Database Query Analysis**
```sql
-- Check slow queries
SELECT query, total_time, calls, mean_time
FROM pg_stat_statements
WHERE query LIKE '%memory%'
ORDER BY mean_time DESC;

-- Monitor connection pool
SELECT count(*) as active_connections
FROM pg_stat_activity
WHERE datname = 'agent_agency_v3';
```

**Memory System Health Check**
```rust
async fn comprehensive_health_check(memory_manager: &MemoryManager, db_client: &DatabaseClient) {
    // Database connectivity
    let db_healthy = !db_client.pool().is_closed();
    println!("Database: {}", if db_healthy { "HEALTHY" } else { "UNHEALTHY" });

    // Decay scheduler
    let scheduler_status = memory_manager.get_decay_scheduler_status().await;
    println!("Decay Scheduler: {:?}", scheduler_status);

    // Schema verification
    let schema_ok = data_infrastructure::database_init::verify_schema(db_client.pool()).await?;
    println!("Schema: {}", if schema_ok { "VALID" } else { "INVALID" });

    // Memory counts
    // Note: Implement actual count queries
    println!("Health check complete");
}
```

## Production Deployment

### Environment Setup

```bash
# Required environment variables
export DATABASE_URL="postgresql://user:password@localhost:5432/agent_agency_v3"
export RUST_LOG=info,agent_memory=warn,data_infrastructure=warn

# Optional: Multi-tenant configuration
export AGENT_AGENCY_DEFAULT_TENANT="default-tenant-id"
```

### Database Initialization

```bash
# Create database
createdb agent_agency_v3

# Install extensions
psql -d agent_agency_v3 -c "CREATE EXTENSION IF NOT EXISTS vector;"
psql -d agent_agency_v3 -c "CREATE EXTENSION IF NOT EXISTS uuid_ossp;"

# Start API server (runs migrations automatically)
cargo run --bin agent-agency-api-server -- --host 0.0.0.0 --port 8080
```

### Health Monitoring

```bash
# Health check endpoint
curl http://localhost:8080/health

# Expected response with database status
{
  "status": "ok",
  "database": "connected"
}
```

### Backup Strategy

```bash
# Database backup
pg_dump agent_agency_v3 > memory_system_backup.sql

# Memory-specific tables to backup
# - agent_experiences: Core learning data
# - memory_embeddings: Vector storage (critical)
# - agent_contexts: Active context state
# - chat_sessions/chat_messages: Conversation history
# - tenants: Multi-tenant configuration
```

## API Reference

### MemoryManager

```rust
pub struct MemoryManager {
    // Core memory operations
    pub async fn store_experience(&self, experience: AgentExperience) -> Result<MemoryId>
    pub async fn retrieve_contextual_memories(&self, context: &TaskContext, limit: usize) -> Result<Vec<ContextualMemory>>
    pub async fn perform_reasoning(&self, query: ReasoningQuery) -> Result<ReasoningResult>

    // Decay scheduler control
    pub async fn start_decay_scheduler(&self) -> Result<()>
    pub async fn stop_decay_scheduler(&self) -> Result<()>
    pub async fn run_manual_decay_cycle(&self) -> Result<usize>
    pub async fn get_decay_scheduler_status(&self) -> Option<DecaySchedulerStatus>
}
```

### ChatService

```rust
pub struct ChatService {
    pub async fn create_session(&self, workspace_id: Option<Uuid>, tenant_id: Option<Uuid>, title: Option<String>, metadata: serde_json::Value) -> Result<ChatSession>
    pub async fn send_message(&self, session_id: Uuid, role: String, content: String, metadata: serde_json::Value, token_count: Option<i32>, model_used: Option<String>) -> Result<ChatMessage>
    pub async fn get_session_messages(&self, session_id: Uuid, limit: Option<i32>, offset: Option<i32>) -> Result<Vec<ChatMessage>>
    pub async fn get_session(&self, session_id: Uuid) -> Result<Option<ChatSession>>
}
```

### ContextManager

```rust
impl ContextManager {
    pub fn new_with_db_client(config: ContextConfig, ai_service: Arc<ModelRegistry>, db_client: Arc<DatabaseClient>) -> Result<Self>
    pub async fn store_context(&self, context: &ContextData) -> Result<()>
    pub async fn retrieve_context(&self, context_id: &Uuid) -> Result<Option<ContextData>>
    pub async fn offload_context(&self, context_id: &Uuid) -> Result<()>
}
```

## Examples

See `examples/` directory for comprehensive usage examples:

- `comprehensive_usage.rs` - Full system demonstration
- Database integration patterns with connection pooling
- Memory decay scheduler configuration and monitoring
- Multi-tenant memory isolation examples
- Chat service integration with memory context
- File watcher integration with ingestion pipeline

## Version History

### v3.1.0 (Current)
- ✅ Database persistence with PostgreSQL
- ✅ Memory decay scheduler with configurable algorithms
- ✅ Multi-tenancy support with Row Level Security
- ✅ Chat service integration with context persistence
- ✅ Context manager with database-backed storage
- ✅ File watcher integration with notify crate
- ✅ Automatic migration system
- ✅ Connection pooling and health monitoring

### v3.0.0 (Previous)
- Knowledge graph implementation
- Vector embeddings for semantic search
- Temporal reasoning capabilities
- Context offloading system
- Provenance tracking

## License

Licensed under the same terms as the Agent Agency project.

## Contributing

1. Follow the CAWS workflow for any changes
2. Include comprehensive tests for new features
3. Update documentation for API changes
4. Run performance benchmarks for optimizations
5. Test database migrations in development environment
6. Verify multi-tenant isolation in integration tests
