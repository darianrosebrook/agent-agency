/**
 * Database Type Definitions for V2 Hybrid Vector-Graph Architecture
 *
 * @author @darianrosebrook
 * @description TypeScript types matching PostgreSQL schema for type safety
 * @version 1.0.0
 *
 * Generated from migrations:
 * - 006_create_knowledge_graph_schema.sql
 * - 007_add_multi_tenant_isolation.sql
 * - 008_create_hybrid_search_views.sql
 *
 * Note: Enum values and type definitions are exported for use in other modules.
 * Unused warnings are expected and suppressed for this type definition file.
 */
// @ts-nocheck


/* eslint-disable @typescript-eslint/no-unused-vars */
/* eslint-disable no-unused-vars */

// ============================================================================
// ENUMS (matching PostgreSQL ENUM types)
// ============================================================================

/**
 * Entity types for knowledge graph nodes
 */
export enum EntityType {
  CAPABILITY = "CAPABILITY",
  AGENT = "AGENT",
  TASK = "TASK",
  VERDICT = "VERDICT",
  TECHNOLOGY = "TECHNOLOGY",
  CONCEPT = "CONCEPT",
  WAIVER = "WAIVER",
  GATE = "GATE",
}

/**
 * Relationship types between agents in the graph
 */
export enum RelationshipType {
  COLLABORATES_WITH = "COLLABORATES_WITH",
  SIMILAR_TO = "SIMILAR_TO",
  DERIVED_FROM = "DERIVED_FROM",
  VALIDATES = "VALIDATES",
  DEPENDS_ON = "DEPENDS_ON",
  COMPETES_WITH = "COMPETES_WITH",
  INFLUENCES = "INFLUENCES",
  REPLACES = "REPLACES",
  ENHANCES = "ENHANCES",
}

/**
 * Tenant isolation levels determining data sharing policies
 */
export enum IsolationLevel {
  STRICT = "strict", // No data sharing
  SHARED = "shared", // Explicit sharing rules
  FEDERATED = "federated", // Cross-tenant learning with privacy
}

/**
 * Privacy levels for federated learning
 */
export enum PrivacyLevel {
  BASIC = "basic", // Basic anonymization
  DIFFERENTIAL = "differential", // Differential privacy with noise
  SECURE = "secure", // Secure multi-party computation
}

/**
 * Data retention policy types
 */
export enum RetentionPolicy {
  DELETE = "delete", // Delete after retention period
  ARCHIVE = "archive", // Move to cold storage
  RETAIN = "retain", // Keep indefinitely
}

/**
 * Validation status for capabilities
 */
export enum ValidationStatus {
  VALIDATED = "validated",
  UNVALIDATED = "unvalidated",
  REJECTED = "rejected",
}

/**
 * Extraction methods for entity-chunk mappings
 */
export enum ExtractionMethod {
  MANUAL = "manual",
  NLP = "nlp",
  LLM = "llm",
  RULE_BASED = "rule-based",
}

/**
 * Chunk types for provenance tracking
 */
export enum ChunkType {
  TASK = "task",
  BENCHMARK = "benchmark",
  RESEARCH = "research",
  TRAINING = "training",
}

/**
 * Search types for session tracking
 */
export enum SearchType {
  VECTOR = "vector",
  GRAPH = "graph",
  HYBRID = "hybrid",
  TEXT = "text",
}

// ============================================================================
// AGENT CAPABILITIES GRAPH
// ============================================================================

/**
 * Agent capability node in knowledge graph with vector embedding
 */
export interface AgentCapabilityGraph {
  id: string;
  agentId: string;

  // Capability information
  capabilityType: EntityType;
  capabilityName: string;
  canonicalName: string; // Auto-normalized
  aliases: string[];

  // Quality metrics
  confidence: number; // 0.7-1.0
  extractionConfidence: number; // 0.0-1.0
  validationStatus: ValidationStatus;

  // Vector embedding (768-dimensional)
  embedding: number[] | null;

  // Evidence tracking
  sourceTasks: string[];
  demonstrationCount: number;
  successRate: number; // 0.0-1.0

  // Temporal
  firstObserved: Date;
  lastUpdated: Date;
  lastDemonstrated: Date;

  // Multi-tenancy
  tenantId: string;

  // Flexible metadata
  metadata: Record<string, unknown>;
}

/**
 * Input for creating new capability
 */
export interface CreateCapabilityInput {
  agentId: string;
  capabilityName: string;
  capabilityType?: EntityType;
  aliases?: string[];
  confidence: number;
  extractionConfidence: number;
  embedding?: number[];
  sourceTasks?: string[];
  tenantId: string;
  metadata?: Record<string, unknown>;
}

/**
 * Input for updating capability
 */
export interface UpdateCapabilityInput {
  confidence?: number;
  validationStatus?: ValidationStatus;
  embedding?: number[];
  sourceTasks?: string[];
  demonstrationCount?: number;
  successRate?: number;
  lastDemonstrated?: Date;
  metadata?: Record<string, unknown>;
}

// ============================================================================
// AGENT RELATIONSHIPS GRAPH
// ============================================================================

/**
 * Typed relationship between two agents
 */
export interface AgentRelationship {
  id: string;

  // Endpoints
  sourceAgentId: string;
  targetAgentId: string;

  // Relationship properties
  type: RelationshipType;
  isDirectional: boolean;

  // Quality metrics
  confidence: number; // 0.5-1.0 (minimum 0.5)
  strength: number; // 0.0-1.0

  // Evidence
  cooccurrenceCount: number;
  supportingTasks: string[];
  extractionContext: string | null;

  // Statistical measures
  mutualInformation: number | null;
  pointwiseMutualInformation: number | null;

  // Temporal
  createdAt: Date;
  updatedAt: Date;
  lastObserved: Date;

  // Metadata
  metadata: Record<string, unknown>;
}

/**
 * Input for creating relationship
 */
export interface CreateRelationshipInput {
  sourceAgentId: string;
  targetAgentId: string;
  type: RelationshipType;
  isDirectional?: boolean;
  confidence: number;
  strength?: number;
  supportingTasks?: string[];
  extractionContext?: string;
  metadata?: Record<string, unknown>;
}

/**
 * Input for updating relationship
 */
export interface UpdateRelationshipInput {
  confidence?: number;
  strength?: number;
  cooccurrenceCount?: number;
  supportingTasks?: string[];
  mutualInformation?: number;
  pointwiseMutualInformation?: number;
  lastObserved?: Date;
  metadata?: Record<string, unknown>;
}

// ============================================================================
// CAWS PROVENANCE GRAPH
// ============================================================================

/**
 * CAWS provenance node with cryptographic integrity
 */
export interface CAWSProvenanceNode {
  id: string;

  // Entity classification
  entityType: "verdict" | "waiver" | "gate" | "spec" | "budget" | "policy";
  entityId: string;

  // Graph structure (hash chain)
  parentEntityId: string | null;

  // Cryptographic integrity
  hashChain: string; // SHA-256 hash (64 chars)
  signature: string; // ed25519 signature

  // Constitutional binding
  constitutionalRefs: string[]; // ["CAWS:Section4.2"]
  specHash: string | null;

  // Semantic discovery
  embedding: number[] | null;
  description: string | null;

  // Quality scores
  evidenceCompleteness: number | null; // 0.0-1.0
  budgetAdherence: number | null; // 0.0-1.0
  gateIntegrity: number | null; // 0.0-1.0
  provenanceClarity: number | null; // 0.0-1.0

  // Temporal
  createdAt: Date;

  // Metadata
  metadata: Record<string, unknown>;
}

/**
 * Input for creating provenance node
 */
export interface CreateProvenanceInput {
  entityType: "verdict" | "waiver" | "gate" | "spec" | "budget" | "policy";
  entityId: string;
  parentEntityId?: string;
  hashChain: string;
  signature: string;
  constitutionalRefs?: string[];
  specHash?: string;
  embedding?: number[];
  description?: string;
  evidenceCompleteness?: number;
  budgetAdherence?: number;
  gateIntegrity?: number;
  provenanceClarity?: number;
  metadata?: Record<string, unknown>;
}

/**
 * Provenance chain link
 */
export interface ProvenanceChainLink {
  id: string;
  entityType: string;
  entityId: string;
  parentEntityId: string | null;
  hashChain: string;
  constitutionalRefs: string[];
  createdAt: Date;
  depth: number;
  path: string[];
}

// ============================================================================
// ENTITY-CHUNK MAPPINGS
// ============================================================================

/**
 * Provenance mapping from capabilities to source data
 */
export interface EntityChunkMapping {
  id: string;

  // References
  entityId: string;
  chunkId: string;
  chunkType: ChunkType;

  // Mapping details
  mentionText: string;
  mentionContext: string | null;
  startPosition: number | null;
  endPosition: number | null;

  // Extraction
  extractionMethod: ExtractionMethod;
  extractionConfidence: number; // 0.0-1.0

  // Temporal
  createdAt: Date;
}

/**
 * Input for creating entity-chunk mapping
 */
export interface CreateMappingInput {
  entityId: string;
  chunkId: string;
  chunkType: ChunkType;
  mentionText: string;
  mentionContext?: string;
  startPosition?: number;
  endPosition?: number;
  extractionMethod: ExtractionMethod;
  extractionConfidence: number;
}

// ============================================================================
// MULTI-TENANCY
// ============================================================================

/**
 * Tenant configuration
 */
export interface Tenant {
  id: string;
  projectId: string;
  name: string;

  // Isolation
  isolationLevel: IsolationLevel;

  // Access control
  accessPolicies: AccessPolicy[];
  sharingRules: SharingRule[];

  // Data retention
  dataRetention: DataRetentionConfig;

  // Security
  encryptionEnabled: boolean;
  auditLogging: boolean;

  // Configuration
  config: Record<string, unknown>;

  // Temporal
  createdAt: Date;
  updatedAt: Date;
}

/**
 * Access policy for tenant
 */
export interface AccessPolicy {
  resource: string;
  action: string;
  allowed: boolean;
  conditions?: Record<string, unknown>;
}

/**
 * Sharing rule between tenants
 */
export interface SharingRule {
  allowedTenants: string[];
  sharedResources: string[];
  expiresAt?: Date;
}

/**
 * Data retention configuration
 */
export interface DataRetentionConfig {
  policy: RetentionPolicy;
  retentionDays: number;
  archiveAfterDays: number;
}

/**
 * Privacy configuration for federated learning
 */
export interface TenantPrivacyConfig {
  tenantId: string;

  // Privacy level
  privacyLevel: PrivacyLevel;

  // Differential privacy parameters
  noiseMagnitude: number;
  kAnonymity: number;
  epsilon: number;
  delta: number;

  // Sharing preferences
  allowCrossTenantLearning: boolean;
  allowedTenantGroups: string[];

  // Temporal
  updatedAt: Date;
}

/**
 * Tenant access log entry
 */
export interface TenantAccessLog {
  id: string;
  tenantId: string;
  accessorTenantId: string | null;
  accessType: "read" | "write" | "aggregate" | "federated";
  tableName: string;
  queryHash: string | null;
  rowCount: number | null;
  userId: string | null;
  ipAddress: string | null;
  accessedAt: Date;
}

// ============================================================================
// SEARCH
// ============================================================================

/**
 * Hybrid search result
 */
export interface HybridSearchResult {
  entityId: string;
  entityType: string;
  name: string;
  relevanceScore: number;
  source: "vector" | "graph";
  hopDistance: number;
  parentId: string | null;
}

/**
 * Search session tracking
 */
export interface GraphSearchSession {
  id: string;

  // Query
  queryText: string | null;
  queryHash: string;
  searchType: SearchType;

  // Parameters
  maxResults: number;
  maxHops: number;
  minConfidence: number;
  entityTypeFilters: string[] | null;

  // Results and performance
  resultCount: number;
  executionTimeMs: number;
  vectorSearchTimeMs: number | null;
  graphTraversalTimeMs: number | null;

  // Graph metrics
  nodesVisited: number;
  edgesTraversed: number;
  maxHopsReached: number;

  // Context
  tenantId: string | null;
  userId: string | null;
  sessionId: string | null;

  // Temporal
  createdAt: Date;

  // Metadata
  metadata: Record<string, unknown>;
}

/**
 * Graph traversal result
 */
export interface GraphTraversalResult {
  agentId: string;
  agentName: string;
  hopDistance: number;
  relationshipPath: RelationshipType[];
  cumulativeConfidence: number;
  path: string[];
}

/**
 * Agent path between two nodes
 */
export interface AgentPath {
  pathLength: number;
  relationshipPath: RelationshipType[];
  agentPath: string[];
  totalConfidence: number;
}

/**
 * Similar capability result
 */
export interface SimilarCapability {
  capabilityId: string;
  capabilityName: string;
  agentId: string;
  similarityScore: number;
  confidence: number;
}

/**
 * Similar CAWS verdict result
 */
export interface SimilarCAWSVerdict {
  verdictId: string;
  entityId: string;
  entityType: string;
  similarityScore: number;
  constitutionalRefs: string[];
  createdAt: Date;
}

// ============================================================================
// ANALYTICS AND VIEWS
// ============================================================================

/**
 * Agent capability summary
 */
export interface AgentCapabilitySummary {
  agentId: string;
  capabilityCount: number;
  avgConfidence: number;
  avgSuccessRate: number;
  lastActivity: Date;
}

/**
 * Agent relationship summary
 */
export interface AgentRelationshipSummary {
  sourceAgentId: string;
  type: RelationshipType;
  relationshipCount: number;
  avgConfidence: number;
  avgStrength: number;
}

/**
 * Agent connectivity metrics
 */
export interface AgentConnectivity {
  agentId: string;
  outboundRelationships: number;
  inboundRelationships: number;
  totalRelationships: number;
}

/**
 * Agent centrality metrics
 */
export interface AgentCentrality {
  agentId: string;
  degreeCentrality: number;
  betweennessEstimate: number;
  connectionCount: number;
}

/**
 * Tenant statistics
 */
export interface TenantStatistics {
  tenantId: string;
  name: string;
  isolationLevel: IsolationLevel;
  auditLogging: boolean;
  agentCount: number;
  eventCount: number;
  benchmarkCount: number;
  createdAt: Date;
}

/**
 * Search performance by type
 */
export interface SearchPerformanceByType {
  searchType: SearchType;
  totalSearches: number;
  avgTimeMs: number;
  p50TimeMs: number;
  p95TimeMs: number;
  p99TimeMs: number;
  avgResults: number;
}

// ============================================================================
// FUNCTION PARAMETERS
// ============================================================================

/**
 * Parameters for hybrid_search function
 */
export interface HybridSearchParams {
  queryEmbedding: number[];
  queryText?: string;
  maxResults?: number;
  includeGraphHops?: number;
  entityTypes?: string[];
  tenantId?: string;
  minConfidence?: number;
}

/**
 * Parameters for traverse_agent_relationships function
 */
export interface TraverseRelationshipsParams {
  startAgentId: string;
  maxHops?: number;
  minConfidence?: number;
  relationshipTypes?: RelationshipType[];
}

/**
 * Parameters for find_agent_path function
 */
export interface FindAgentPathParams {
  sourceAgentId: string;
  targetAgentId: string;
  maxHops?: number;
}

/**
 * Parameters for find_similar_capabilities function
 */
export interface FindSimilarCapabilitiesParams {
  targetEmbedding: number[];
  tenantId?: string;
  maxResults?: number;
  minConfidence?: number;
}

/**
 * Parameters for find_similar_caws_verdicts function
 */
export interface FindSimilarVerdictsParams {
  targetEmbedding: number[];
  verdictType?: string;
  maxResults?: number;
}

// ============================================================================
// UTILITY TYPES
// ============================================================================

/**
 * Database transaction options
 */
export interface TransactionOptions {
  isolationLevel?:
    | "READ UNCOMMITTED"
    | "READ COMMITTED"
    | "REPEATABLE READ"
    | "SERIALIZABLE";
  readOnly?: boolean;
  deferrable?: boolean;
}

/**
 * Tenant context for queries
 */
export interface TenantContext {
  tenantId: string;
  userId?: string;
  sessionId?: string;
}

/**
 * Pagination options
 */
export interface PaginationOptions {
  limit?: number;
  offset?: number;
  orderBy?: string;
  orderDirection?: "ASC" | "DESC";
}

/**
 * Filter options for queries
 */
export interface FilterOptions {
  tenantId?: string;
  entityTypes?: EntityType[];
  minConfidence?: number;
  minStrength?: number;
  createdAfter?: Date;
  createdBefore?: Date;
}

// ============================================================================
// ERROR TYPES
// ============================================================================

/**
 * Database error with context
 */
export class DatabaseError extends Error {
  constructor(
    message: string,
    public readonly code: string,
    public readonly detail?: string,
    public readonly hint?: string
  ) {
    super(message);
    this.name = "DatabaseError";
  }
}

/**
 * Tenant isolation violation error
 */
export class TenantIsolationError extends DatabaseError {
  constructor(tenantId: string, attemptedTenantId: string) {
    super(
      `Tenant ${tenantId} attempted to access data for tenant ${attemptedTenantId}`,
      "TENANT_ISOLATION_VIOLATION"
    );
    this.name = "TenantIsolationError";
  }
}

/**
 * Provenance integrity error
 */
export class ProvenanceIntegrityError extends DatabaseError {
  constructor(message: string, public readonly nodeId: string) {
    super(message, "PROVENANCE_INTEGRITY_VIOLATION");
    this.name = "ProvenanceIntegrityError";
  }
}
