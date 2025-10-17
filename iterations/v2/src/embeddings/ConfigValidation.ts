/**
 * @fileoverview Configuration Validation for Embedding Services
 *
 * Provides comprehensive validation and environment-specific configuration
 * management for production deployment safety.
 *
 * @author @darianrosebrook
 */

import { EmbeddingConfig } from "./types.js";

/**
 * Validation result
 */
export interface ValidationResult {
  isValid: boolean;
  errors: ValidationError[];
  warnings: ValidationWarning[];
}

/**
 * Validation error
 */
export interface ValidationError {
  field: string;
  code: string;
  message: string;
  value?: any;
}

/**
 * Validation warning
 */
export interface ValidationWarning {
  field: string;
  code: string;
  message: string;
  suggestion?: string;
}

/**
 * Environment-specific configuration profiles
 */
export interface ConfigProfile {
  name: string;
  description: string;
  config: Partial<EmbeddingConfig>;
  validationRules: ValidationRule[];
}

/**
 * Validation rule
 */
export interface ValidationRule {
  field: keyof EmbeddingConfig;
  required?: boolean;
  min?: number;
  max?: number;
  pattern?: RegExp;
  allowedValues?: any[];
  customValidator?: (
    value: any,
    config: EmbeddingConfig
  ) => ValidationError | null;
}

/**
 * Configuration validator
 */
export class EmbeddingConfigValidator {
  private static readonly DEFAULT_RULES: ValidationRule[] = [
    {
      field: "ollamaEndpoint",
      required: true,
      pattern: /^https?:\/\/[^\s/$.?#].[^\s]*$/i,
    },
    {
      field: "cacheSize",
      min: 10,
      max: 100000,
    },
    {
      field: "timeout",
      min: 1000, // 1 second
      max: 300000, // 5 minutes
    },
    {
      field: "rateLimitPerSecond",
      min: 1,
      max: 1000,
    },
    {
      field: "model",
      allowedValues: [
        "embeddinggemma",
        "gemma3n:e2b",
        "gemma3n:e4b",
        "all-minilm",
        "text-embedding-ada-002",
      ],
    },
  ];

  /**
   * Validate embedding configuration
   */
  static validate(config: EmbeddingConfig): ValidationResult {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];

    // Apply default validation rules
    for (const rule of this.DEFAULT_RULES) {
      const result = this.validateField(config, rule);
      if (result.error) {
        errors.push(result.error);
      }
      if (result.warning) {
        warnings.push(result.warning);
      }
    }

    // Cross-field validations
    const crossFieldErrors = this.validateCrossField(config);
    errors.push(...crossFieldErrors);

    return {
      isValid: errors.length === 0,
      errors,
      warnings,
    };
  }

  /**
   * Validate single field against rule
   */
  private static validateField(
    config: EmbeddingConfig,
    rule: ValidationRule
  ): { error?: ValidationError; warning?: ValidationWarning } {
    const value = config[rule.field];
    const fieldName = rule.field as string;

    // Required field check
    if (
      rule.required &&
      (value === undefined || value === null || value === "")
    ) {
      return {
        error: {
          field: fieldName,
          code: "REQUIRED_FIELD_MISSING",
          message: `Required field '${fieldName}' is missing or empty`,
          value,
        },
      };
    }

    // Skip further validation if field is not required and not provided
    if (!rule.required && (value === undefined || value === null)) {
      return {};
    }

    // Type validation based on field
    const typeError = this.validateFieldType(fieldName, value);
    if (typeError) {
      return { error: typeError };
    }

    // Range validation
    if (typeof value === "number") {
      if (rule.min !== undefined && value < rule.min) {
        return {
          error: {
            field: fieldName,
            code: "VALUE_TOO_LOW",
            message: `Field '${fieldName}' value ${value} is below minimum ${rule.min}`,
            value,
          },
        };
      }

      if (rule.max !== undefined && value > rule.max) {
        return {
          error: {
            field: fieldName,
            code: "VALUE_TOO_HIGH",
            message: `Field '${fieldName}' value ${value} is above maximum ${rule.max}`,
            value,
          },
        };
      }
    }

    // Pattern validation
    if (
      rule.pattern &&
      typeof value === "string" &&
      !rule.pattern.test(value)
    ) {
      return {
        error: {
          field: fieldName,
          code: "INVALID_FORMAT",
          message: `Field '${fieldName}' value '${value}' does not match required pattern`,
          value,
        },
      };
    }

    // Allowed values validation
    if (rule.allowedValues && !rule.allowedValues.includes(value)) {
      return {
        warning: {
          field: fieldName,
          code: "VALUE_NOT_RECOMMENDED",
          message: `Field '${fieldName}' value '${value}' is not in the recommended list: ${rule.allowedValues.join(
            ", "
          )}`,
          suggestion: `Consider using one of: ${rule.allowedValues.join(", ")}`,
        },
      };
    }

    // Custom validation
    if (rule.customValidator) {
      const customError = rule.customValidator(value, config);
      if (customError) {
        return { error: customError };
      }
    }

    return {};
  }

  /**
   * Validate field type
   */
  private static validateFieldType(
    field: string,
    value: any
  ): ValidationError | null {
    switch (field) {
      case "ollamaEndpoint":
        if (typeof value !== "string") {
          return {
            field,
            code: "INVALID_TYPE",
            message: `Field '${field}' must be a string`,
            value,
          };
        }
        break;

      case "cacheSize":
      case "timeout":
      case "rateLimitPerSecond":
        if (typeof value !== "number" || !Number.isInteger(value)) {
          return {
            field,
            code: "INVALID_TYPE",
            message: `Field '${field}' must be an integer`,
            value,
          };
        }
        break;

      case "model":
        if (typeof value !== "string") {
          return {
            field,
            code: "INVALID_TYPE",
            message: `Field '${field}' must be a string`,
            value,
          };
        }
        break;
    }

    return null;
  }

  /**
   * Cross-field validations
   */
  private static validateCrossField(
    config: EmbeddingConfig
  ): ValidationError[] {
    const errors: ValidationError[] = [];

    // Timeout should be reasonable for rate limiting
    if (config.timeout && config.rateLimitPerSecond) {
      const expectedResponseTime = 1000 / config.rateLimitPerSecond; // ms per request
      if (config.timeout < expectedResponseTime * 2) {
        errors.push({
          field: "timeout",
          code: "TIMEOUT_TOO_LOW",
          message: `Timeout ${config.timeout}ms may be too low for rate limit of ${config.rateLimitPerSecond} requests/second`,
          value: config.timeout,
        });
      }
    }

    // Cache size should be reasonable for memory usage
    if (config.cacheSize && config.cacheSize > 50000) {
      errors.push({
        field: "cacheSize",
        code: "CACHE_SIZE_TOO_LARGE",
        message: `Cache size ${config.cacheSize} may cause excessive memory usage`,
        value: config.cacheSize,
      });
    }

    return errors;
  }

  /**
   * Load configuration from environment variables
   */
  static fromEnvironment(): EmbeddingConfig {
    return {
      ollamaEndpoint:
        process.env.OLLAMA_HOST ||
        process.env.OLLAMA_ENDPOINT ||
        "http://localhost:11434",
      model: process.env.EMBEDDING_MODEL || "embeddinggemma",
      cacheSize: process.env.EMBEDDING_CACHE_SIZE
        ? parseInt(process.env.EMBEDDING_CACHE_SIZE)
        : undefined,
      timeout: process.env.EMBEDDING_TIMEOUT
        ? parseInt(process.env.EMBEDDING_TIMEOUT)
        : undefined,
      rateLimitPerSecond: process.env.EMBEDDING_RATE_LIMIT
        ? parseInt(process.env.EMBEDDING_RATE_LIMIT)
        : undefined,
    };
  }

  /**
   * Get configuration profiles for different environments
   */
  static getProfiles(): Record<string, ConfigProfile> {
    return {
      development: {
        name: "development",
        description: "Development environment with relaxed settings",
        config: {
          ollamaEndpoint: "http://localhost:11434",
          cacheSize: 100,
          timeout: 30000,
          rateLimitPerSecond: 10,
        },
        validationRules: [
          ...this.DEFAULT_RULES,
          {
            field: "cacheSize",
            max: 1000, // Smaller cache for dev
          },
        ],
      },

      staging: {
        name: "staging",
        description: "Staging environment with moderate settings",
        config: {
          ollamaEndpoint: process.env.OLLAMA_HOST || "http://localhost:11434",
          cacheSize: 1000,
          timeout: 45000,
          rateLimitPerSecond: 25,
        },
        validationRules: this.DEFAULT_RULES,
      },

      production: {
        name: "production",
        description: "Production environment with strict settings",
        config: {
          ollamaEndpoint: process.env.OLLAMA_HOST!,
          cacheSize: 5000,
          timeout: 60000,
          rateLimitPerSecond: 50,
        },
        validationRules: [
          ...this.DEFAULT_RULES,
          {
            field: "ollamaEndpoint",
            required: true,
            pattern: /^https:\/\/[^\s/$.?#].[^\s]*$/i, // HTTPS required for prod
          },
          {
            field: "cacheSize",
            min: 1000,
            max: 50000,
          },
          {
            field: "rateLimitPerSecond",
            min: 10,
            max: 200,
          },
        ],
      },

      highPerformance: {
        name: "high-performance",
        description: "High-performance environment for heavy workloads",
        config: {
          ollamaEndpoint: process.env.OLLAMA_HOST!,
          cacheSize: 20000,
          timeout: 120000,
          rateLimitPerSecond: 200,
        },
        validationRules: [
          ...this.DEFAULT_RULES,
          {
            field: "cacheSize",
            min: 5000,
            max: 100000,
          },
          {
            field: "rateLimitPerSecond",
            min: 50,
            max: 500,
          },
        ],
      },

      resourceConstrained: {
        name: "resource-constrained",
        description: "Minimal resource usage for constrained environments",
        config: {
          ollamaEndpoint: process.env.OLLAMA_HOST || "http://localhost:11434",
          cacheSize: 50,
          timeout: 90000, // Longer timeout for slow networks
          rateLimitPerSecond: 2,
        },
        validationRules: [
          ...this.DEFAULT_RULES,
          {
            field: "cacheSize",
            max: 200,
          },
          {
            field: "rateLimitPerSecond",
            max: 5,
          },
        ],
      },
    };
  }

  /**
   * Load configuration profile
   */
  static loadProfile(profileName: string): EmbeddingConfig {
    const profiles = this.getProfiles();
    const profile = profiles[profileName];

    if (!profile) {
      throw new Error(`Unknown configuration profile: ${profileName}`);
    }

    // Merge with environment variables
    const envConfig = this.fromEnvironment();
    return { ...profile.config, ...envConfig };
  }

  /**
   * Validate configuration against profile
   */
  static validateProfile(
    config: EmbeddingConfig,
    profileName: string
  ): ValidationResult {
    const profiles = this.getProfiles();
    const profile = profiles[profileName];

    if (!profile) {
      return {
        isValid: false,
        errors: [
          {
            field: "profile",
            code: "UNKNOWN_PROFILE",
            message: `Unknown configuration profile: ${profileName}`,
          },
        ],
        warnings: [],
      };
    }

    // Apply profile-specific validation rules
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];

    for (const rule of profile.validationRules) {
      const result = this.validateField(config, rule);
      if (result.error) {
        errors.push(result.error);
      }
      if (result.warning) {
        warnings.push(result.warning);
      }
    }

    return {
      isValid: errors.length === 0,
      errors,
      warnings,
    };
  }

  /**
   * Get configuration template with documentation
   */
  static getConfigTemplate(): string {
    return `
# Embedding Service Configuration
# Copy this to your environment or config file

# Ollama API endpoint (required)
OLLAMA_HOST=http://localhost:11434

# Embedding model to use
EMBEDDING_MODEL=embeddinggemma

# Cache size for embeddings (optional, default: 1000)
EMBEDDING_CACHE_SIZE=1000

# Request timeout in milliseconds (optional, default: 30000)
EMBEDDING_TIMEOUT=30000

# Rate limit in requests per second (optional, default: 50)
EMBEDDING_RATE_LIMIT=50

# Logging level (optional, default: INFO)
LOG_LEVEL=INFO
`;
  }
}

/**
 * Configuration validation error
 */
export class ConfigurationError extends Error {
  public readonly errors: ValidationError[];
  public readonly warnings: ValidationWarning[];

  constructor(
    message: string,
    errors: ValidationError[],
    warnings: ValidationWarning[] = []
  ) {
    super(message);
    this.name = "ConfigurationError";
    this.errors = errors;
    this.warnings = warnings;
  }
}
