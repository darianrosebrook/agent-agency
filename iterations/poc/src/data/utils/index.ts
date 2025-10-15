/**
 * @fileoverview Data Layer Utilities
 * @author @darianrosebrook
 *
 * Common utilities and helper functions for the data layer.
 * Provides validation, formatting, and data transformation utilities.
 */

/**
 * Generate a unique ID with timestamp and random component
 */
export function generateId(prefix: string = "id"): string {
  const timestamp = Date.now();
  const random = Math.random().toString(36).substring(2, 9);
  return `${prefix}_${timestamp}_${random}`;
}

/**
 * Validate tenant ID format
 */
export function isValidTenantId(tenantId: string): boolean {
  return (
    typeof tenantId === "string" &&
    tenantId.length > 0 &&
    tenantId.length <= 255 &&
    /^[a-zA-Z0-9_-]+$/.test(tenantId)
  );
}

/**
 * Validate entity ID format
 */
export function isValidEntityId(entityId: string): boolean {
  return (
    typeof entityId === "string" &&
    entityId.length > 0 &&
    entityId.length <= 255 &&
    /^[a-zA-Z0-9_-]+$/.test(entityId)
  );
}

/**
 * Sanitize string input for database queries
 */
export function sanitizeString(
  input: string,
  maxLength: number = 1000
): string {
  if (typeof input !== "string") {
    return "";
  }

  return input
    .trim()
    .substring(0, maxLength)
    .replace(/[\x00-\x1F\x7F-\x9F]/g, ""); // Remove control characters
}

/**
 * Validate JSON data
 */
export function isValidJSON(data: any): boolean {
  try {
    JSON.stringify(data);
    return true;
  } catch {
    return false;
  }
}

/**
 * Deep clone an object safely
 */
export function deepClone<T>(obj: T): T {
  if (obj === null || typeof obj !== "object") {
    return obj;
  }

  if (obj instanceof Date) {
    return new Date(obj.getTime()) as unknown as T;
  }

  if (Array.isArray(obj)) {
    return obj.map((item) => deepClone(item)) as unknown as T;
  }

  const cloned = {} as T;
  for (const key in obj) {
    if (obj.hasOwnProperty(key)) {
      cloned[key] = deepClone(obj[key]);
    }
  }

  return cloned;
}

/**
 * Calculate hash for data integrity checks
 */
export async function calculateHash(data: any): Promise<string> {
  const crypto = await import("crypto");
  const str = JSON.stringify(data);
  return crypto.default.createHash("sha256").update(str).digest("hex");
}

/**
 * Format database query results for consistent API
 */
export function formatQueryResult<T>(
  success: boolean,
  data?: T,
  error?: string,
  duration?: number,
  queryId?: string
): {
  success: boolean;
  data?: T;
  error?: string;
  duration: number;
  queryId: string;
} {
  return {
    success,
    data,
    error,
    duration: duration || 0,
    queryId: queryId || generateId("query"),
  };
}

/**
 * Validate database configuration
 */
export function validateDatabaseConfig(config: any): {
  valid: boolean;
  errors: string[];
} {
  const errors: string[] = [];

  if (!config.host || typeof config.host !== "string") {
    errors.push("Database host is required and must be a string");
  }

  if (
    !config.port ||
    typeof config.port !== "number" ||
    config.port < 1 ||
    config.port > 65535
  ) {
    errors.push("Database port must be a number between 1 and 65535");
  }

  if (!config.database || typeof config.database !== "string") {
    errors.push("Database name is required and must be a string");
  }

  if (!config.username || typeof config.username !== "string") {
    errors.push("Database username is required and must be a string");
  }

  if (config.password !== undefined && typeof config.password !== "string") {
    errors.push("Database password must be a string if provided");
  }

  if (
    config.maxConnections !== undefined &&
    (typeof config.maxConnections !== "number" || config.maxConnections < 1)
  ) {
    errors.push("Max connections must be a number greater than 0");
  }

  return {
    valid: errors.length === 0,
    errors,
  };
}

/**
 * Validate cache configuration
 */
export function validateCacheConfig(config: any): {
  valid: boolean;
  errors: string[];
} {
  const errors: string[] = [];

  if (!config.host || typeof config.host !== "string") {
    errors.push("Cache host is required and must be a string");
  }

  if (
    !config.port ||
    typeof config.port !== "number" ||
    config.port < 1 ||
    config.port > 65535
  ) {
    errors.push("Cache port must be a number between 1 and 65535");
  }

  if (config.password !== undefined && typeof config.password !== "string") {
    errors.push("Cache password must be a string if provided");
  }

  if (
    config.db !== undefined &&
    (typeof config.db !== "number" || config.db < 0)
  ) {
    errors.push("Cache database must be a non-negative number if provided");
  }

  return {
    valid: errors.length === 0,
    errors,
  };
}

/**
 * Create database URL from configuration
 */
export function createDatabaseUrl(config: any): string {
  const protocol = config.ssl ? "postgresql" : "postgresql";
  const auth = config.password
    ? `${encodeURIComponent(config.username)}:${encodeURIComponent(
        config.password
      )}`
    : encodeURIComponent(config.username);

  return `${protocol}://${auth}@${config.host}:${config.port}/${config.database}`;
}

/**
 * Create cache URL from configuration
 */
export function createCacheUrl(config: any): string {
  const protocol = "redis";
  const auth = config.password
    ? `:${encodeURIComponent(config.password)}@`
    : "";

  return `${protocol}://${auth}${config.host}:${config.port}${
    config.db ? `/${config.db}` : ""
  }`;
}

/**
 * Sleep utility for testing and retries
 */
export function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * Retry utility with exponential backoff
 */
export async function retry<T>(
  fn: () => Promise<T>,
  maxAttempts: number = 3,
  baseDelay: number = 1000,
  maxDelay: number = 10000
): Promise<T> {
  let lastError: Error;

  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    try {
      return await fn();
    } catch (error) {
      lastError = error as Error;

      if (attempt === maxAttempts) {
        break;
      }

      const delay = Math.min(baseDelay * Math.pow(2, attempt - 1), maxDelay);
      await sleep(delay);
    }
  }

  throw lastError!;
}

/**
 * Batch process items with concurrency control
 */
export async function batchProcess<T, R>(
  items: T[],
  processor: (item: T) => Promise<R>,
  batchSize: number = 10,
  concurrency: number = 3
): Promise<R[]> {
  const results: R[] = [];

  for (let i = 0; i < items.length; i += batchSize) {
    const batch = items.slice(i, i + batchSize);
    const batchPromises = batch.map(processor);

    // Process batches with concurrency limit
    const batchResults = await Promise.all(
      batchPromises
        .reduce((acc: Promise<R>[][], promise) => {
          if (acc.length === 0 || acc[acc.length - 1].length >= concurrency) {
            acc.push([]);
          }
          acc[acc.length - 1].push(promise);
          return acc;
        }, [])
        .map((group) => Promise.all(group))
    );

    results.push(...batchResults.flat());
  }

  return results;
}
