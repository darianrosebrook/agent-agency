/**
 * @fileoverview Data Layer Exports
 * @author @darianrosebrook
 *
 * Unified exports for the entire data layer module.
 * Provides easy access to all data layer components.
 */

// Core data layer
export { DataLayer } from "./DataLayer";

// Connection management
export { PostgreSQLConnection } from "./connection/PostgreSQLConnection";

// Caching
export { MultiLevelCache } from "./cache/MultiLevelCache";
export { RedisCache } from "./cache/RedisCache";

// DAOs
export { AgentDAO } from "./dao/AgentDAO";
export { BaseDAO } from "./dao/BaseDAO";
export { ExperienceDAO } from "./dao/ExperienceDAO";
export { VectorDAO } from "./dao/VectorDAO";

// Monitoring
export { PerformanceMonitor } from "./monitoring/PerformanceMonitor";

// Security
export * from "./security";

// Migrations
export { MigrationManager } from "./migrations/MigrationManager";

// Types and interfaces
export * from "./types";

// Utils
export * from "./utils";
