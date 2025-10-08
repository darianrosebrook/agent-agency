# Multi-Tenant Memory System Database Schema

This directory contains the database migrations for the multi-tenant memory system, designed for PostgreSQL with pgvector support.

## Overview

The database schema supports a complete multi-tenant memory system with the following key features:

- **Tenant Isolation**: Secure data isolation between projects and tenants
- **Context Offloading**: Efficient storage of compressed LLM contexts
- **Federated Learning**: Privacy-preserving cross-tenant intelligence sharing
- **Vector Search**: Semantic similarity search using pgvector
- **Audit Logging**: Comprehensive operation tracking
- **Performance Monitoring**: Built-in metrics collection

## Architecture

### Core Tables

```
Projects (1) ─── (N) Tenants (1) ─── (N) Memories
    │                    │
    │                    ├── Access Policies
    │                    ├── Sharing Rules
    │                    └── Retention Policies
    │
    └── Federated Sessions
         └── Aggregated Insights
```

### Key Components

1. **Projects**: Top-level organizational units
2. **Tenants**: Project-specific execution contexts with isolation levels
3. **Contextual Memories**: Core memory storage with vector embeddings
4. **Offloaded Contexts**: Compressed external context storage
5. **Federated Learning**: Cross-tenant intelligence sharing
6. **Audit & Monitoring**: Operation tracking and performance metrics

## Migration Files

### 001_create_multi_tenant_schema.sql
- Initial schema creation
- Core tables for tenants, memories, and federated learning
- Basic indexes and constraints
- Row Level Security (RLS) setup
- Utility views for common queries

### 002_add_performance_optimizations.sql
- Additional performance indexes
- Materialized views for analytics
- Utility functions for maintenance
- Advanced constraints and validations

## Setup Instructions

### Prerequisites

1. PostgreSQL 13+ with pgvector extension
2. Database user with schema creation privileges

### Installation

1. **Enable pgvector extension:**
   ```sql
   CREATE EXTENSION IF NOT EXISTS "pgvector";
   CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
   ```

2. **Run migrations in order:**
   ```bash
   psql -d your_database -f migrations/001_create_multi_tenant_schema.sql
   psql -d your_database -f migrations/002_add_performance_optimizations.sql
   ```

3. **Configure Row Level Security (Optional):**
   Uncomment and modify the RLS policies in the migration file based on your authentication system.

## Table Reference

### Core Tables

| Table | Purpose | Key Fields |
|-------|---------|------------|
| `projects` | Top-level organizational units | `id`, `name`, `owner_id` |
| `tenants` | Project-specific execution contexts | `tenant_id`, `project_id`, `isolation_level` |
| `contextual_memories` | Core memory storage | `memory_id`, `relevance_score`, `content` |
| `offloaded_contexts` | Compressed context storage | `context_id`, `embedding`, `compression_ratio` |
| `federated_sessions` | Cross-tenant learning sessions | `session_id`, `topic`, `status` |

### Supporting Tables

| Table | Purpose |
|-------|---------|
| `tenant_access_policies` | Access control rules |
| `tenant_sharing_rules` | Cross-tenant sharing configuration |
| `data_retention_policies` | Data lifecycle management |
| `memory_relationships` | Knowledge graph connections |
| `federated_participants` | Federated learning participants |
| `aggregated_insights` | Results from federated learning |
| `audit_log` | Operation tracking |
| `performance_metrics` | System performance data |

## Key Features

### Tenant Isolation

- **Isolation Levels**: `strict`, `shared`, `federated`
- **Access Policies**: Granular resource-level permissions
- **Sharing Rules**: Controlled cross-tenant data sharing

### Vector Search & Embeddings

- **pgvector Integration**: High-performance vector similarity search
- **Embedding Storage**: 384-dimension embeddings (configurable)
- **Similarity Search**: Cosine similarity with IVF indexing

### Federated Learning

- **Privacy Preservation**: Differential privacy and anonymization
- **Reputation System**: Participant trustworthiness scoring
- **Session Management**: Coordinated learning across tenants

### Performance & Monitoring

- **Materialized Views**: Pre-computed analytics
- **Performance Metrics**: Built-in monitoring and alerting
- **Audit Logging**: Complete operation traceability

## Usage Examples

### Basic Tenant Setup

```sql
-- Create a project
INSERT INTO projects (name, description, owner_id)
VALUES ('my-project', 'My AI project', 'user123');

-- Create a tenant
INSERT INTO tenants (tenant_id, project_id, name, isolation_level)
VALUES ('tenant-001', (SELECT id FROM projects WHERE name = 'my-project'), 'Production', 'shared');
```

### Storing Memories

```sql
-- Store a contextual memory
INSERT INTO contextual_memories (
    memory_id, tenant_id, relevance_score, content
) VALUES (
    'memory-123',
    (SELECT id FROM tenants WHERE tenant_id = 'tenant-001'),
    0.85,
    '{"taskType": "learning", "outcome": "success", "lessons": ["Important lesson"]}'
);
```

### Vector Similarity Search

```sql
-- Find similar memories using vector search
SELECT memory_id, relevance_score,
       1 - (embedding <=> '[0.1, 0.2, ...]') as similarity
FROM offloaded_contexts
ORDER BY embedding <=> '[0.1, 0.2, ...]'
LIMIT 10;
```

### Federated Learning

```sql
-- Register as federated learning participant
INSERT INTO federated_participants (tenant_id, privacy_level)
VALUES ((SELECT id FROM tenants WHERE tenant_id = 'tenant-001'), 'differential');

-- Create a learning session
INSERT INTO federated_sessions (session_id, topic, initiator_tenant_id)
VALUES ('session-abc', 'code-review-best-practices', (SELECT id FROM tenants WHERE tenant_id = 'tenant-001'));
```

## Maintenance

### Regular Tasks

1. **Refresh Analytics Views:**
   ```sql
   SELECT refresh_analytics_views();
   ```

2. **Clean Expired Data:**
   ```sql
   SELECT clean_expired_memories();
   ```

3. **Archive Old Audit Logs:**
   ```sql
   SELECT archive_old_audit_logs(365);
   ```

### Performance Monitoring

```sql
-- Check tenant memory usage
SELECT * FROM get_tenant_memory_usage((SELECT id FROM tenants WHERE tenant_id = 'tenant-001'));

-- View system performance
SELECT * FROM system_performance_summary
WHERE hour > NOW() - INTERVAL '24 hours'
ORDER BY hour DESC;
```

## Security Considerations

### Row Level Security (RLS)

The schema includes RLS policies (commented out) that should be enabled based on your authentication system. These ensure tenants can only access their own data.

### Data Encryption

- Enable `encryption_enabled` on tenants requiring data encryption
- Use PostgreSQL's built-in encryption or external encryption services
- Consider encrypting sensitive metadata fields

### Audit Compliance

- All operations are logged in the `audit_log` table
- Retention policies can be configured per tenant
- Regular audit log archival is recommended

## Performance Tuning

### Indexes

The schema includes comprehensive indexing:
- B-tree indexes for equality and range queries
- GIN indexes for JSONB and full-text search
- IVFFlat indexes for vector similarity search

### Partitioning

Consider partitioning large tables:
- `audit_log` by month
- `performance_metrics` by time range
- `contextual_memories` by tenant (if needed)

### Monitoring

Use the provided performance monitoring functions:
```sql
-- Log custom metrics
SELECT log_performance_metric(tenant_id, 'memory', 'retrieval_time', 150.5, 'ms');

-- Check system health
SELECT * FROM system_health
WHERE timestamp > NOW() - INTERVAL '1 hour'
ORDER BY timestamp DESC;
```

## Migration Strategy

When deploying updates:

1. **Test migrations** in a staging environment
2. **Backup data** before running migrations
3. **Run migrations** in a maintenance window
4. **Update application code** to use new schema features
5. **Monitor performance** after deployment

## Troubleshooting

### Common Issues

1. **pgvector not available**: Ensure the extension is installed
2. **RLS blocking queries**: Check RLS policies and user permissions
3. **Slow vector searches**: Adjust IVF parameters or add more indexes
4. **Memory usage**: Monitor and clean expired data regularly

### Performance Tips

- Use `EXPLAIN ANALYZE` to optimize slow queries
- Consider connection pooling for high-traffic applications
- Monitor index usage and rebuild if necessary
- Archive old data to maintain performance

## Future Enhancements

- **Time-series partitioning** for better performance
- **Advanced compression** for offloaded contexts
- **Machine learning** for automated insight aggregation
- **Real-time analytics** with streaming data processing
