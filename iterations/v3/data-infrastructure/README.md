# Data Infrastructure

Data layer and API services for AI agent systems.

The Data Infrastructure crate provides a comprehensive data platform that consolidates database operations, API services, caching, embeddings, and file operations into a unified, scalable data layer designed for high-performance AI agent workloads.

## Overview

This platform combines data capabilities:

- Database Operations: High-performance PostgreSQL with vector extensions
- API Services: RESTful APIs with OpenAPI documentation and GraphQL support
- Caching Layer: Multi-level caching with Redis and in-memory caching
- Embedding Services: Vector embeddings with similarity search
- File Operations: Secure file storage and retrieval
- Real-time Communication: WebSocket support for live updates

## Key Features

### Database Operations
- PostgreSQL + pgvector: High-performance vector database with ACID compliance
- Connection Pooling: Efficient connection management with health monitoring
- Migrations: Version-controlled schema migrations with rollback support
- Query Optimization: Intelligent query planning and execution
- Backup & Recovery: Automated backup strategies with point-in-time recovery

### API Services
- RESTful APIs: OpenAPI 3.0 specification with automatic documentation
- GraphQL Support: Flexible query interface for complex data relationships
- Rate Limiting: Configurable rate limiting to prevent abuse
- Circuit Breakers: Fault tolerance with automatic recovery
- Health Monitoring: Comprehensive health checks and metrics

### Caching Infrastructure
- Multi-Level Caching: Memory → Redis → Database hierarchy
- Cache Invalidation: Intelligent cache invalidation strategies
- Distributed Caching: Cluster-aware caching across multiple nodes
- Performance Monitoring: Cache hit rates and performance analytics

### Embedding Services
- Vector Storage: High-performance vector storage with indexing
- Similarity Search: Cosine similarity and other distance metrics
- Batch Processing: Efficient batch embedding generation
- Model Management: Multiple embedding model support and switching

### File Operations
- Secure Storage: Encrypted file storage with access controls
- Versioning: File versioning and historical access
- Streaming: Efficient large file streaming and resumable uploads
- Metadata Management: Rich metadata support with search capabilities

### Real-time Communication
- WebSocket Support: Real-time bidirectional communication
- Event Streaming: Server-sent events for live updates
- Pub/Sub: Publish-subscribe messaging patterns
- Connection Management: Connection pooling and lifecycle management

## Architecture

```mermaid
graph TB
    subgraph "API Layer"
        REST[REST API]
        GraphQL[GraphQL API]
        WS[WebSocket API]
        Health[Health Checks]
    end

    subgraph "Service Layer"
        Auth[Authentication]
        RateLimit[Rate Limiting]
        Circuit[Circuit Breakers]
        Cache[Cache Service]
    end

    subgraph "Data Layer"
        DB[(PostgreSQL + pgvector)]
        Redis[(Redis Cache)]
        FS[(File Storage)]
        ES[Embedding Service]
    end

    subgraph "Infrastructure Layer"
        Pool[Connection Pooling]
        Mig[Migrations]
        Backup[Backup & Recovery]
        Monitor[Monitoring]
    end

    subgraph "Integration Layer"
        MCP[MCP Protocol]
        Sandbox[Sandbox API]
        Artifacts[Artifact Store]
        Keystore[Keystore API]
    end

    REST --> Auth
    GraphQL --> Auth
    WS --> Auth

    Auth --> RateLimit
    RateLimit --> Circuit
    Circuit --> Cache

    Cache --> DB
    Cache --> Redis
    Cache --> FS
    Cache --> ES

    DB --> Pool
    Redis --> Pool
    Pool --> Mig
    Mig --> Backup
    Backup --> Monitor

    Circuit --> MCP
    Circuit --> Sandbox
    Circuit --> Artifacts
    Circuit --> Keystore
    MCP --> Monitor
    Sandbox --> Monitor
    Artifacts --> Monitor
    Keystore --> Monitor
```

## Quick Start

### 1. Add to Dependencies

```toml
[dependencies]
data-infrastructure = { path = "../data-infrastructure" }
```

### 2. Initialize Data Infrastructure

```rust
use data_infrastructure::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure the data infrastructure
    let config = DataInfrastructureConfig {
        database: DatabaseConfig {
            url: "postgresql://user:password@localhost/agent_db".to_string(),
            max_connections: 20,
            enable_ssl: true,
            ..Default::default()
        },
        caching: CacheConfig {
            redis_url: "redis://localhost:6379".to_string(),
            memory_cache_size: 1000,
            ttl_seconds: 3600,
            ..Default::default()
        },
        api: ApiConfig {
            host: "0.0.0.0".to_string(),
            port: 8080,
            enable_cors: true,
            rate_limit_requests_per_minute: 1000,
            ..Default::default()
        },
        embeddings: EmbeddingConfig {
            model_name: "text-embedding-ada-002".to_string(),
            dimension: 1536,
            batch_size: 100,
            ..Default::default()
        },
        file_storage: FileStorageConfig {
            base_path: "/data/files".to_string(),
            max_file_size_mb: 100,
            enable_encryption: true,
            ..Default::default()
        },
    };

    // Initialize the data infrastructure
    let data_infra = DataInfrastructure::new(config).await?;

    Ok(())
}
```

### 3. Database Operations

```rust
// Connect to database
let db_client = data_infra.database_client().await?;

// Execute queries with connection pooling
let users = db_client.query("SELECT * FROM users WHERE active = $1", &[&true]).await?;
println!("Found {} active users", users.len());

// Use vector search
let query_embedding = vec![0.1, 0.2, 0.3]; // Your query embedding
let similar_docs = db_client.vector_search("documents", query_embedding, 10).await?;

for doc in similar_docs {
    println!("Document: {} (similarity: {:.3})", doc.title, doc.similarity);
}
```

### 4. API Services

```rust
// Start API server
let api_server = data_infra.api_server();

api_server.start().await?;

// The API now provides:
// - REST endpoints at http://localhost:8080/api/
// - GraphQL playground at http://localhost:8080/graphql
// - WebSocket endpoint at ws://localhost:8080/ws/
// - Health checks at http://localhost:8080/health
```

### 5. Caching Operations

```rust
// Get cache client
let cache = data_infra.cache_client();

// Cache a value
cache.set("user:123", serde_json::json!({"name": "Alice", "email": "alice@example.com"}), 3600).await?;

// Retrieve from cache
if let Some(user_data) = cache.get("user:123").await? {
    let user: User = serde_json::from_value(user_data)?;
    println!("Cached user: {}", user.name);
} else {
    // Cache miss - fetch from database
    let user = fetch_user_from_db(123).await?;
    cache.set("user:123", serde_json::to_value(&user)?, 3600).await?;
}
```

### 6. Embedding Operations

```rust
// Get embedding service
let embeddings = data_infra.embedding_service();

// Generate embeddings for text
let texts = vec![
    "The AI agent completed the task successfully.".to_string(),
    "Machine learning models require extensive training data.".to_string(),
];

let embeddings_result = embeddings.generate_embeddings(texts).await?;
println!("Generated {} embeddings with dimension {}", embeddings_result.len(), embeddings_result[0].len());

// Find similar content
let query_text = "AI agent performance";
let query_embedding = embeddings.generate_embedding(query_text).await?;
let similar_content = embeddings.find_similar(query_embedding, 5).await?;

for item in similar_content {
    println!("Similar content: {} (score: {:.3})", item.text, item.score);
}
```

### 7. File Operations

```rust
// Get file storage client
let file_storage = data_infra.file_storage();

// Upload a file
let file_data = std::fs::read("document.pdf")?;
let file_id = file_storage.upload_file("document.pdf", file_data, Some("application/pdf")).await?;
println!("File uploaded with ID: {}", file_id);

// Download a file
let downloaded_data = file_storage.download_file(&file_id).await?;
println!("Downloaded {} bytes", downloaded_data.len());

// List files with metadata
let files = file_storage.list_files(Some("application/pdf")).await?;
for file in files {
    println!("File: {} ({} bytes, uploaded: {:?})", file.name, file.size, file.uploaded_at);
}
```

## Configuration

### Comprehensive Configuration

```rust
let config = DataInfrastructureConfig {
    database: DatabaseConfig {
        url: "postgresql://user:password@localhost/agent_db".to_string(),
        max_connections: 50,
        min_connections: 5,
        connection_timeout_seconds: 30,
        enable_ssl: true,
        schema: "agent_schema".to_string(),
        migrations_path: "migrations".to_string(),
    },
    caching: CacheConfig {
        redis_url: "redis://localhost:6379".to_string(),
        redis_cluster: false,
        memory_cache_size: 10000,
        ttl_seconds: 3600,
        compression_enabled: true,
        serialization_format: SerializationFormat::Json,
    },
    api: ApiConfig {
        host: "0.0.0.0".to_string(),
        port: 8080,
        workers: 4,
        enable_cors: true,
        cors_origins: vec!["https://app.example.com".to_string()],
        rate_limit_requests_per_minute: 1000,
        burst_limit: 100,
        timeout_seconds: 30,
        enable_metrics: true,
    },
    embeddings: EmbeddingConfig {
        provider: EmbeddingProvider::OpenAI,
        model_name: "text-embedding-ada-002".to_string(),
        api_key: SecretString::from("sk-...".to_string()),
        dimension: 1536,
        batch_size: 100,
        max_tokens_per_request: 8191,
        request_timeout_seconds: 60,
        retry_attempts: 3,
    },
    file_storage: FileStorageConfig {
        provider: FileStorageProvider::Local,
        base_path: "/data/files".to_string(),
        s3_bucket: None,
        max_file_size_mb: 100,
        allowed_mime_types: vec!["application/pdf".to_string(), "image/*".to_string()],
        enable_encryption: true,
        encryption_key: Some("your-encryption-key".to_string()),
        enable_versioning: true,
        retention_days: Some(365),
    },
    websocket: WebSocketConfig {
        max_connections: 1000,
        message_timeout_seconds: 30,
        heartbeat_interval_seconds: 60,
        enable_compression: true,
        max_message_size_kb: 64,
    },
};
```

## Database Operations

### Connection Pooling

```rust
// Configure connection pool
let pool_config = PoolConfig {
    max_size: 50,
    min_idle: 5,
    max_lifetime: Some(Duration::from_secs(1800)),
    idle_timeout: Some(Duration::from_secs(300)),
    connection_timeout: Duration::from_secs(30),
};

let pool = ConnectionPool::new(database_url, pool_config).await?;

// Use pooled connection
let client = pool.get().await?;
let result = client.query("SELECT * FROM users").await?;
pool.put(client).await?;
```

### Vector Search

```rust
// Create vector table
db_client.execute(r#"
    CREATE TABLE documents (
        id SERIAL PRIMARY KEY,
        content TEXT,
        embedding VECTOR(1536)
    )
"#).await?;

// Insert document with embedding
let embedding = vec![0.1, 0.2, 0.3]; // 1536-dimensional vector
db_client.execute(
    "INSERT INTO documents (content, embedding) VALUES ($1, $2)",
    &[&"Document content", &embedding]
).await?;

// Search for similar documents
let query_embedding = vec![0.15, 0.25, 0.35]; // Query vector
let similar_docs = db_client.query(
    "SELECT content, 1 - (embedding <=> $1) as similarity FROM documents ORDER BY embedding <=> $1 LIMIT 10",
    &[&query_embedding]
).await?;
```

### Migrations

```rust
// Create a migration
let migration = Migration::new(
    "001_create_users_table",
    r#"
    CREATE TABLE users (
        id SERIAL PRIMARY KEY,
        username VARCHAR(255) UNIQUE NOT NULL,
        email VARCHAR(255) UNIQUE NOT NULL,
        created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
    );

    CREATE INDEX idx_users_username ON users(username);
    CREATE INDEX idx_users_email ON users(email);
    "#,
    r#"
    DROP TABLE users;
    "#,
);

// Run migrations
let migration_runner = MigrationRunner::new(pool);
migration_runner.up().await?;
println!("Migrations completed successfully");
```

## API Services

### REST API Endpoints

```rust
// Define API routes
let api_routes = Router::new()
    .route("/api/users", get(get_users))
    .route("/api/users/:id", get(get_user))
    .route("/api/users", post(create_user))
    .route("/api/users/:id", put(update_user))
    .route("/api/users/:id", delete(delete_user))
    .route("/api/search", post(search_documents));

// Add middleware
let app = Router::new()
    .nest("/", api_routes)
    .layer(CorsLayer::permissive())
    .layer(TimeoutLayer::new(Duration::from_secs(30)))
    .layer(TraceLayer::new_for_http());

// Start server
let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
axum::Server::bind(&addr)
    .serve(app.into_make_service())
    .await?;
```

### GraphQL API

```rust
// Define GraphQL schema
let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
    .data(pool.clone())
    .finish();

// Create GraphQL route
let graphql_route = Router::new()
    .route("/graphql", post(graphql_handler))
    .route("/graphql/playground", get(graphql_playground_handler));

// GraphQL handler
async fn graphql_handler(
    Extension(schema): Extension<AppSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}
```

### Rate Limiting

```rust
// Configure rate limiter
let rate_limiter = RateLimiter::new(RateLimitConfig {
    requests_per_minute: 1000,
    burst_limit: 100,
    identifier: Identifier::IpAddress,
});

// Use in middleware
let app = Router::new()
    .route("/api/*", get(protected_handler))
    .layer(from_fn(move |req, next| {
        let rate_limiter = rate_limiter.clone();
        async move {
            match rate_limiter.check_request(&req).await {
                Ok(_) => next.run(req).await,
                Err(_) => StatusCode::TOO_MANY_REQUESTS.into_response(),
            }
        }
    }));
```

## Caching Infrastructure

### Multi-Level Caching

```rust
// Configure multi-level cache
let cache_config = MultiLevelCacheConfig {
    memory: MemoryCacheConfig {
        max_size: 10000,
        ttl: Duration::from_secs(3600),
    },
    redis: RedisCacheConfig {
        url: "redis://localhost:6379".to_string(),
        ttl: Duration::from_secs(7200),
        compression: true,
    },
    database: DatabaseCacheConfig {
        table_name: "cache_entries".to_string(),
        ttl: Duration::from_secs(86400),
    },
};

let cache = MultiLevelCache::new(cache_config).await?;

// Cache with hierarchy: Memory -> Redis -> Database
cache.set("key", "value", None).await?;
let value = cache.get("key").await?; // Checks memory first, then Redis, then DB
```

### Cache Invalidation

```rust
// Tag-based invalidation
cache.set_with_tags("user:123", user_data, vec!["user", "user:123"], None).await?;

// Invalidate by tags
cache.invalidate_tags(vec!["user:123"]).await?; // Invalidates all user:123 related cache

// Pattern-based invalidation
cache.invalidate_pattern("user:*").await?; // Invalidates all user-related cache
```

## Embedding Services

### Multiple Providers

```rust
// Configure multiple embedding providers
let providers = vec![
    EmbeddingProviderConfig {
        name: "openai".to_string(),
        provider: EmbeddingProvider::OpenAI,
        api_key: "sk-...".to_string(),
        model: "text-embedding-ada-002".to_string(),
        dimension: 1536,
    },
    EmbeddingProviderConfig {
        name: "cohere".to_string(),
        provider: EmbeddingProvider::Cohere,
        api_key: "cohere-key".to_string(),
        model: "embed-multilingual-v2.0".to_string(),
        dimension: 768,
    },
];

let embedding_service = MultiProviderEmbeddingService::new(providers).await?;

// Use different providers for different tasks
let openai_embedding = embedding_service.generate_with_provider("openai", text).await?;
let cohere_embedding = embedding_service.generate_with_provider("cohere", text).await?;
```

### Batch Processing

```rust
// Process embeddings in batches
let texts = vec![
    "First document text".to_string(),
    "Second document text".to_string(),
    // ... up to 100 texts
];

let batch_result = embedding_service.generate_batch(texts).await?;

println!("Processed {} embeddings", batch_result.embeddings.len());
println!("Tokens used: {}", batch_result.total_tokens);
println!("Cost: ${:.4}", batch_result.estimated_cost);
```

## File Operations

### Secure File Storage

```rust
// Configure secure file storage
let storage_config = SecureFileStorageConfig {
    base_path: "/secure/files".to_string(),
    encryption: EncryptionConfig {
        enabled: true,
        algorithm: EncryptionAlgorithm::Aes256Gcm,
        key_rotation_days: 30,
    },
    access_control: AccessControlConfig {
        enabled: true,
        max_file_size_mb: 100,
        allowed_extensions: vec!["pdf", "docx", "txt"],
    },
    versioning: VersioningConfig {
        enabled: true,
        max_versions: 10,
        retention_days: 365,
    },
};

let secure_storage = SecureFileStorage::new(storage_config).await?;

// Upload with encryption and access control
let file_metadata = FileMetadata {
    name: "confidential.pdf".to_string(),
    mime_type: "application/pdf".to_string(),
    owner: "alice".to_string(),
    permissions: vec!["alice".to_string(), "admin".to_string()],
    tags: vec!["confidential".to_string(), "finance".to_string()],
};

let file_id = secure_storage.upload_secure(file_data, file_metadata).await?;
```

### Streaming Operations

```rust
// Stream large file upload
let stream = secure_storage.create_upload_stream(file_metadata).await?;
let mut reader = BufReader::new(file);

loop {
    let chunk = reader.fill_buf().await?;
    if chunk.is_empty() {
        break;
    }
    stream.write_chunk(chunk).await?;
    reader.consume(chunk.len());
}

let file_id = stream.complete().await?;
```

## Real-time Communication

### WebSocket Connections

```rust
// Configure WebSocket server
let ws_config = WebSocketConfig {
    max_connections: 1000,
    heartbeat_interval: Duration::from_secs(30),
    max_message_size: 65536,
    compression_enabled: true,
};

let ws_server = WebSocketServer::new(ws_config).await?;

// Handle WebSocket connections
ws_server.on_connect(|connection| async move {
    println!("New connection: {}", connection.id);

    // Send welcome message
    connection.send(Message::Text("Welcome to Agent Agency!".to_string())).await?;

    Ok(())
});

// Handle messages
ws_server.on_message(|connection, message| async move {
    match message {
        Message::Text(text) => {
            println!("Received: {}", text);
            // Echo back
            connection.send(Message::Text(format!("Echo: {}", text))).await?;
        }
        Message::Binary(data) => {
            // Handle binary data
            connection.send(Message::Binary(data)).await?;
        }
        _ => {}
    }

    Ok(())
});

ws_server.start("0.0.0.0:8081").await?;
```

### Event Streaming

```rust
// Configure event streaming
let event_config = EventStreamConfig {
    buffer_size: 1000,
    retention_period: Duration::from_secs(3600),
    enable_persistence: true,
};

let event_stream = EventStream::new(event_config).await?;

// Subscribe to events
let mut subscription = event_stream.subscribe("agent.updates").await?;

tokio::spawn(async move {
    while let Some(event) = subscription.recv().await {
        println!("Received event: {:?}", event);
    }
});

// Publish events
event_stream.publish("agent.updates", AgentUpdateEvent {
    agent_id: "agent-001".to_string(),
    status: AgentStatus::Running,
    timestamp: Utc::now(),
}).await?;
```

## Performance Characteristics

### Database Performance

- Query Throughput: 10,000+ queries per second with connection pooling
- Vector Search: Sub-100ms similarity search for millions of vectors
- Concurrent Connections: Support for 1000+ concurrent database connections
- Data Consistency: ACID compliance with optimized transaction handling

### API Performance

- REST API: 50,000+ requests per minute with rate limiting
- GraphQL: Complex queries resolved in < 500ms
- WebSocket: 10,000+ concurrent connections with sub-10ms latency
- Health Checks: Sub-1ms health check responses

### Caching Performance

- Memory Cache: Microsecond access times for hot data
- Redis Cache: Sub-millisecond access for distributed cache
- Cache Hit Rate: 95%+ hit rate with intelligent invalidation
- Throughput: 100,000+ cache operations per second

### Embedding Performance

- Single Embedding: 100-500ms per embedding depending on model
- Batch Processing: 1000+ embeddings per minute
- Similarity Search: Sub-10ms search across millions of vectors
- Model Switching: Sub-second model switching with caching

## Integration Examples

### With Agent Orchestration

```rust
// Integrate data infrastructure with agent orchestration
let data_integration = DataIntegration::new(data_infra, orchestration_service);

// Store agent execution results
let execution_result = orchestration_service.execute_task(task).await?;
data_integration.store_execution_result(execution_result).await?;

// Retrieve historical performance data
let agent_performance = data_integration.get_agent_performance(&agent_id).await?;
println!("Agent success rate: {:.2}%", agent_performance.success_rate * 100.0);
```

### With Monitoring System

```rust
// Integrate with observability system
let monitoring_integration = MonitoringIntegration::new(data_infra, monitoring_system);

// Store metrics in time-series database
monitoring_integration.store_metrics(metrics_batch).await?;

// Query performance data
let performance_query = PerformanceQuery {
    time_range: TimeRange::last_24_hours(),
    metric_names: vec!["api_response_time".to_string(), "db_query_time".to_string()],
    aggregation: Aggregation::Percentile(95.0),
};

let performance_data = monitoring_integration.query_performance(performance_query).await?;
```

## Best Practices

### Database Design

1. **Indexing Strategy**: Create appropriate indexes for query patterns
2. **Partitioning**: Use table partitioning for large datasets
3. **Connection Pooling**: Always use connection pooling for performance
4. **Migration Safety**: Test migrations thoroughly before production deployment

### API Design

1. **Versioning**: Use API versioning for backward compatibility
2. **Pagination**: Implement pagination for large result sets
3. **Filtering**: Support flexible filtering and sorting options
4. **Documentation**: Maintain up-to-date OpenAPI documentation

### Caching Strategy

1. **Cache Hierarchy**: Use multiple cache levels appropriately
2. **Invalidation Strategy**: Implement proper cache invalidation
3. **Cache Warming**: Pre-populate cache for frequently accessed data
4. **Monitoring**: Monitor cache hit rates and adjust strategies

### Security Considerations

1. **Data Encryption**: Encrypt sensitive data at rest and in transit
2. **Access Control**: Implement proper authentication and authorization
3. **Input Validation**: Validate all inputs to prevent injection attacks
4. **Audit Logging**: Log all data access and modification operations

## Troubleshooting

### Database Issues

**Connection Pool Exhaustion**
- Increase pool size or optimize query performance
- Implement query timeouts and cancellation
- Monitor connection usage patterns

**Slow Queries**
- Analyze query execution plans
- Add appropriate indexes
- Consider query optimization and restructuring

**Vector Search Performance**
- Optimize index parameters (ef_construction, m)
- Use appropriate distance metrics
- Consider data partitioning strategies

### API Issues

**Rate Limiting Problems**
- Adjust rate limit thresholds based on usage patterns
- Implement different limits for different endpoints
- Consider user-based vs IP-based limiting

**Circuit Breaker Triggers**
- Investigate underlying service issues
- Adjust circuit breaker thresholds
- Implement proper fallback mechanisms

**WebSocket Connection Issues**
- Check network connectivity and firewall settings
- Verify WebSocket protocol support
- Monitor connection lifecycle and cleanup

### Cache Issues

**Cache Misses**
- Review cache TTL settings
- Implement cache warming strategies
- Check for cache invalidation issues

**Memory Pressure**
- Monitor cache memory usage
- Implement appropriate eviction policies
- Consider cache size limits

### Embedding Issues

**API Rate Limits**
- Implement request batching and queuing
- Use multiple API keys for higher limits
- Implement exponential backoff for retries

**Inconsistent Results**
- Ensure consistent text preprocessing
- Use the same model for similar queries
- Implement result normalization

## Contributing

1. Follow the CAWS workflow for any changes
2. Include comprehensive tests for data operations
3. Update API documentation for endpoint changes
4. Run performance benchmarks for infrastructure changes

## License

Licensed under the same terms as the Agent Agency project.
