use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Context preservation engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPreservationConfig {
    /// Context storage configuration
    pub storage: ContextStorageConfig,
    /// Multi-tenant configuration
    pub multi_tenant: MultiTenantConfig,
    /// Context synthesis configuration
    pub synthesis: ContextSynthesisConfig,
    /// Performance configuration
    pub performance: PerformanceConfig,
    /// Integration configuration
    pub integration: IntegrationConfig,
    /// Encryption configuration
    pub encryption: EncryptionConfig,
}

/// Context storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextStorageConfig {
    /// Maximum context size (bytes)
    pub max_context_size: u64,
    /// Context retention period (hours)
    pub retention_hours: u32,
    /// Maximum number of contexts per tenant
    pub max_contexts_per_tenant: u32,
    /// Enable persistent storage
    pub enable_persistent_storage: bool,
    /// Enable in-memory caching
    pub enable_memory_cache: bool,
    /// Cache size limit (bytes)
    pub cache_size_limit: u64,
    /// Enable compression
    pub enable_compression: bool,
    /// Enable differential storage
    pub enable_differential_storage: bool,
    /// Compression level (1-9)
    pub compression_level: u32,
    /// Maximum snapshot size in MB
    pub max_snapshot_size_mb: u32,
    /// Enable checksum validation
    pub checksum_validation: bool,
}

/// Multi-tenant configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiTenantConfig {
    /// Enable multi-tenant support
    pub enabled: bool,
    /// Default tenant ID
    pub default_tenant_id: String,
    /// Tenant isolation level
    pub isolation_level: TenantIsolationLevel,
    /// Cross-tenant context sharing
    pub allow_cross_tenant_sharing: bool,
    /// Tenant-specific context limits
    pub tenant_limits: HashMap<String, TenantLimits>,
}

/// Tenant isolation level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TenantIsolationLevel {
    /// Strict isolation - no cross-tenant access
    Strict,
    /// Partial isolation - limited cross-tenant access
    Partial,
    /// Shared - full cross-tenant access
    Shared,
}

/// Tenant-specific limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantLimits {
    /// Maximum contexts per tenant
    pub max_contexts: u32,
    /// Maximum context size (bytes)
    pub max_context_size: u64,
    /// Context retention period (hours)
    pub retention_hours: u32,
    /// Maximum concurrent operations
    pub max_concurrent_operations: u32,
}

/// Context synthesis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSynthesisConfig {
    /// Enable context synthesis
    pub enabled: bool,
    /// Synthesis similarity threshold
    pub similarity_threshold: f64,
    /// Maximum synthesis depth
    pub max_synthesis_depth: u32,
    /// Enable cross-reference detection
    pub enable_cross_references: bool,
    /// Maximum cross-references per context
    pub max_cross_references: u32,
    /// Synthesis timeout (seconds)
    pub synthesis_timeout: u64,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Enable performance monitoring
    pub enable_monitoring: bool,
    /// Performance metrics retention (hours)
    pub metrics_retention_hours: u32,
    /// Enable performance optimization
    pub enable_optimization: bool,
    /// Optimization interval (seconds)
    pub optimization_interval: u64,
    /// Enable adaptive caching
    pub enable_adaptive_caching: bool,
}

/// Integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationConfig {
    /// Research agent integration
    pub research_agent: ResearchAgentIntegration,
    /// Council integration
    pub council: CouncilIntegration,
    /// Worker pool integration
    pub worker_pool: WorkerPoolIntegration,
    /// Security integration
    pub security: SecurityIntegration,
}

/// Research agent integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchAgentIntegration {
    /// Enable research agent integration
    pub enabled: bool,
    /// Research agent endpoint
    pub endpoint: String,
    /// Request timeout (seconds)
    pub timeout: u64,
    /// Enable context sharing with research agent
    pub enable_context_sharing: bool,
}

/// Council integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilIntegration {
    /// Enable council integration
    pub enabled: bool,
    /// Council endpoint
    pub endpoint: String,
    /// Request timeout (seconds)
    pub timeout: u64,
    /// Enable context sharing with council
    pub enable_context_sharing: bool,
}

/// Worker pool integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerPoolIntegration {
    /// Enable worker pool integration
    pub enabled: bool,
    /// Worker pool endpoint
    pub endpoint: String,
    /// Request timeout (seconds)
    pub timeout: u64,
    /// Enable context sharing with worker pool
    pub enable_context_sharing: bool,
}

/// Security integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIntegration {
    /// Enable security integration
    pub enabled: bool,
    /// Security policy enforcer endpoint
    pub endpoint: String,
    /// Request timeout (seconds)
    pub timeout: u64,
    /// Enable context validation
    pub enable_context_validation: bool,
}

/// Context preservation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPreservationResult {
    /// Result ID
    pub id: Uuid,
    /// Whether context was preserved
    pub preserved: bool,
    /// Context ID
    pub context_id: Uuid,
    /// Tenant ID
    pub tenant_id: String,
    /// Context size (bytes)
    pub context_size: u64,
    /// Preservation time (milliseconds)
    pub preservation_time_ms: u64,
    /// Context metadata
    pub metadata: ContextMetadata,
    /// Preservation statistics
    pub statistics: PreservationStatistics,
}

/// Context metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMetadata {
    /// Context type
    pub context_type: ContextType,
    /// Context priority
    pub priority: ContextPriority,
    /// Context tags
    pub tags: Vec<String>,
    /// Context description
    pub description: String,
    /// Context source
    pub source: String,
    /// Context version
    pub version: String,
    /// Context dependencies
    pub dependencies: Vec<Uuid>,
    /// Context relationships
    pub relationships: Vec<ContextRelationship>,
}

/// Context type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContextType {
    /// Task context
    Task,
    /// Worker context
    Worker,
    /// Council context
    Council,
    /// Research context
    Research,
    /// Security context
    Security,
    /// Performance context
    Performance,
    /// User context
    User,
    /// System context
    System,
    /// Other context
    Other,
}

/// Context priority
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContextPriority {
    /// Low priority
    Low,
    /// Medium priority
    Medium,
    /// High priority
    High,
    /// Critical priority
    Critical,
}

/// Context relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextRelationship {
    /// Related context ID
    pub related_context_id: Uuid,
    /// Relationship type
    pub relationship_type: RelationshipType,
    /// Relationship strength
    pub strength: f64,
    /// Relationship description
    pub description: String,
}

/// Relationship type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RelationshipType {
    /// Parent-child relationship
    ParentChild,
    /// Sibling relationship
    Sibling,
    /// Dependency relationship
    Dependency,
    /// Reference relationship
    Reference,
    /// Similarity relationship
    Similarity,
    /// Other relationship
    Other,
}

/// Preservation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreservationStatistics {
    /// Total contexts preserved
    pub total_contexts: u64,
    /// Successful preservations
    pub successful_preservations: u64,
    /// Failed preservations
    pub failed_preservations: u64,
    /// Average preservation time (milliseconds)
    pub avg_preservation_time_ms: f64,
    /// Context reuse rate
    pub context_reuse_rate: f64,
    /// Cross-reference rate
    pub cross_reference_rate: f64,
    /// Last updated
    pub last_updated: DateTime<Utc>,
}

/// Context preservation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPreservationRequest {
    /// Request ID
    pub id: Uuid,
    /// Tenant ID
    pub tenant_id: String,
    /// Context data
    pub context_data: ContextData,
    /// Context metadata
    pub metadata: ContextMetadata,
    /// Preservation options
    pub options: PreservationOptions,
    /// Request timestamp
    pub timestamp: DateTime<Utc>,
}

/// Context data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextData {
    /// Context content
    pub content: String,
    /// Context format
    pub format: ContextFormat,
    /// Context encoding
    pub encoding: String,
    /// Context compression
    pub compression: Option<CompressionInfo>,
    /// Context encryption
    pub encryption: Option<EncryptionInfo>,
    /// Context checksum
    pub checksum: String,
}

/// Context format
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContextFormat {
    /// JSON format
    Json,
    /// YAML format
    Yaml,
    /// Text format
    Text,
    /// Binary format
    Binary,
    /// Other format
    Other,
}

/// Compression information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionInfo {
    /// Compression algorithm
    pub algorithm: String,
    /// Compression ratio
    pub ratio: f64,
    /// Original size (bytes)
    pub original_size: u64,
    /// Compressed size (bytes)
    pub compressed_size: u64,
}

/// Preservation options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreservationOptions {
    /// Enable compression
    pub enable_compression: bool,
    /// Enable encryption
    pub enable_encryption: bool,
    /// Enable cross-referencing
    pub enable_cross_referencing: bool,
    /// Enable synthesis
    pub enable_synthesis: bool,
    /// Retention period (hours)
    pub retention_hours: Option<u32>,
    /// Priority level
    pub priority: ContextPriority,
}

/// Context retrieval request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextRetrievalRequest {
    /// Request ID
    pub id: Uuid,
    /// Tenant ID
    pub tenant_id: String,
    /// Context ID
    pub context_id: Uuid,
    /// Retrieval options
    pub options: RetrievalOptions,
    /// Request timestamp
    pub timestamp: DateTime<Utc>,
}

/// Retrieval options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalOptions {
    /// Include metadata
    pub include_metadata: bool,
    /// Include relationships
    pub include_relationships: bool,
    /// Include cross-references
    pub include_cross_references: bool,
    /// Include synthesis
    pub include_synthesis: bool,
    /// Decompress if needed
    pub decompress: bool,
    /// Decrypt if needed
    pub decrypt: bool,
}

/// Context retrieval result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextRetrievalResult {
    /// Result ID
    pub id: Uuid,
    /// Whether context was found
    pub found: bool,
    /// Context data
    pub context_data: Option<ContextData>,
    /// Context metadata
    pub metadata: Option<ContextMetadata>,
    /// Context relationships
    pub relationships: Vec<ContextRelationship>,
    /// Cross-references
    pub cross_references: Vec<CrossReference>,
    /// Synthesis results
    pub synthesis_results: Vec<SynthesisResult>,
    /// Retrieval time (milliseconds)
    pub retrieval_time_ms: u64,
}

/// Cross-reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossReference {
    /// Cross-reference ID
    pub id: Uuid,
    /// Referenced context ID
    pub referenced_context_id: Uuid,
    /// Reference type
    pub reference_type: ReferenceType,
    /// Reference strength
    pub strength: f64,
    /// Reference context
    pub context: String,
}

/// Reference type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReferenceType {
    /// Direct reference
    Direct,
    /// Indirect reference
    Indirect,
    /// Similarity reference
    Similarity,
    /// Dependency reference
    Dependency,
    /// Other reference
    Other,
}

/// Synthesis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesisResult {
    /// Synthesis ID
    pub id: Uuid,
    /// Synthesized context ID
    pub synthesized_context_id: Uuid,
    /// Synthesis type
    pub synthesis_type: SynthesisType,
    /// Synthesis confidence
    pub confidence: f64,
    /// Synthesis description
    pub description: String,
}

/// Synthesis type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SynthesisType {
    /// Context aggregation
    Aggregation,
    /// Context summarization
    Summarization,
    /// Context transformation
    Transformation,
    /// Context enrichment
    Enrichment,
    /// Other synthesis
    Other,
}

/// Context preservation engine statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPreservationStats {
    /// Total requests processed
    pub total_requests: u64,
    /// Successful preservations
    pub successful_preservations: u64,
    /// Failed preservations
    pub failed_preservations: u64,
    /// Total retrievals
    pub total_retrievals: u64,
    /// Successful retrievals
    pub successful_retrievals: u64,
    /// Failed retrievals
    pub failed_retrievals: u64,
    /// Average preservation time (milliseconds)
    pub avg_preservation_time_ms: f64,
    /// Average retrieval time (milliseconds)
    pub avg_retrieval_time_ms: f64,
    /// Context reuse rate
    pub context_reuse_rate: f64,
    /// Cross-reference rate
    pub cross_reference_rate: f64,
    /// Synthesis rate
    pub synthesis_rate: f64,
    /// Last updated
    pub last_updated: DateTime<Utc>,
}

/// Context snapshot for differential storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSnapshot {
    /// Snapshot ID
    pub id: String,
    /// Session ID
    pub session_id: String,
    /// Iteration number
    pub iteration_number: u32,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Original size in bytes
    pub original_size: u64,
    /// Compressed size in bytes
    pub compressed_size: u64,
    /// Compression ratio
    pub compression_ratio: f64,
    /// Whether this is a differential snapshot
    pub is_diff: bool,
    /// Base snapshot ID (for diff snapshots)
    pub based_on_snapshot_id: Option<String>,
    /// SHA256 checksum
    pub checksum: Option<String>,
    /// Compressed context data
    pub compressed_data: Vec<u8>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Context restoration result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextRestorationResult {
    /// Snapshot ID
    pub snapshot_id: String,
    /// Success flag
    pub success: bool,
    /// Restored context (if successful)
    pub context: Option<serde_json::Value>,
    /// Time taken to restore (ms)
    pub restoration_time_ms: u64,
    /// Error message (if failed)
    pub error: Option<String>,
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextCacheStats {
    /// Total number of snapshots
    pub total_snapshots: usize,
    /// Total size in bytes
    pub total_size_bytes: u64,
    /// Average compression ratio
    pub avg_compression_ratio: f64,
    /// Number of base snapshots
    pub base_snapshots_count: usize,
    /// Number of active sessions
    pub sessions_count: usize,
}

/// Context performance data for learning system integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPerformanceData {
    /// Effectiveness score (0.0 to 1.0)
    pub effectiveness_score: f64,
    /// Utilization rate (0.0 to 1.0)
    pub utilization_rate: f64,
    /// Freshness score (0.0 to 1.0)
    pub freshness_score: f64,
}

/// Encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// Enable encryption
    pub enabled: bool,
    /// Encryption algorithm
    pub algorithm: EncryptionAlgorithm,
    /// Key derivation function
    pub key_derivation: KeyDerivationFunction,
    /// Key rotation interval (hours)
    pub key_rotation_interval_hours: u32,
    /// Enable key caching
    pub enable_key_caching: bool,
    /// Key cache TTL (seconds)
    pub key_cache_ttl_seconds: u64,
    /// Enable audit logging
    pub enable_audit_logging: bool,
    /// Maximum key age (hours)
    pub max_key_age_hours: u32,
}

/// Encryption algorithm
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EncryptionAlgorithm {
    /// AES-256-GCM
    Aes256Gcm,
    /// AES-256-CBC
    Aes256Cbc,
    /// ChaCha20-Poly1305
    ChaCha20Poly1305,
}

/// Key derivation function
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum KeyDerivationFunction {
    /// PBKDF2 with SHA-256
    Pbkdf2Sha256,
    /// Argon2id
    Argon2id,
    /// Scrypt
    Scrypt,
}

/// Encryption metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionInfo {
    /// Encryption algorithm used
    pub algorithm: EncryptionAlgorithm,
    /// Key ID used for encryption
    pub key_id: String,
    /// Initialization vector (IV)
    pub iv: Vec<u8>,
    /// Authentication tag (for AEAD)
    pub auth_tag: Option<Vec<u8>>,
    /// Key derivation salt
    pub salt: Vec<u8>,
    /// Encryption timestamp
    pub encrypted_at: DateTime<Utc>,
    /// Key version
    pub key_version: u32,
}

/// Key management information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyInfo {
    /// Key ID
    pub key_id: String,
    /// Key version
    pub key_version: u32,
    /// Key algorithm
    pub algorithm: EncryptionAlgorithm,
    /// Key creation time
    pub created_at: DateTime<Utc>,
    /// Key expiration time
    pub expires_at: Option<DateTime<Utc>>,
    /// Key status
    pub status: KeyStatus,
    /// Key usage count
    pub usage_count: u64,
    /// Last used time
    pub last_used_at: Option<DateTime<Utc>>,
}

/// Key status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum KeyStatus {
    /// Active key
    Active,
    /// Rotated key (still valid for decryption)
    Rotated,
    /// Expired key
    Expired,
    /// Revoked key
    Revoked,
}

/// Encryption audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionAuditLog {
    /// Log entry ID
    pub id: Uuid,
    /// Operation type
    pub operation: EncryptionOperation,
    /// Key ID used
    pub key_id: String,
    /// Context ID (if applicable)
    pub context_id: Option<Uuid>,
    /// Tenant ID
    pub tenant_id: String,
    /// Operation timestamp
    pub timestamp: DateTime<Utc>,
    /// Operation result
    pub result: OperationResult,
    /// Error message (if failed)
    pub error_message: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Encryption operation type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EncryptionOperation {
    /// Key generation
    KeyGeneration,
    /// Key rotation
    KeyRotation,
    /// Data encryption
    DataEncryption,
    /// Data decryption
    DataDecryption,
    /// Key revocation
    KeyRevocation,
}

/// Operation result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OperationResult {
    /// Success
    Success,
    /// Failure
    Failure,
    /// Partial success
    PartialSuccess,
}
