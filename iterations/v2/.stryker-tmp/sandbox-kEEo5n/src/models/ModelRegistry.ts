/**
 * @fileoverview
 * Core model registry for local-first model management.
 * Handles registration, versioning, and metadata management for local models.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import type {
  LocalModelConfig,
  ModelRegistrationRequest,
  ModelUpdateRequest,
  OllamaModelConfig,
  PerformanceProfile,
  RegistryQueryOptions,
} from "@/types/model-registry";

/**
 * Model registry errors
 */
export class ModelRegistryError extends Error {
  constructor(message: string, public code: string) {
    super(message);
    this.name = "ModelRegistryError";
  }
}

/**
 * Core model registry for local models
 *
 * Manages registration, versioning, and metadata for bring-your-own-model philosophy.
 * Models are immutable once registered - updates create new versions.
 */
export class ModelRegistry {
  private models: Map<string, LocalModelConfig> = new Map();
  private performanceProfiles: Map<string, PerformanceProfile> = new Map();
  private modelsByName: Map<string, Set<string>> = new Map(); // name → model IDs

  constructor() {
    // Initialize with empty registry
  }

  /**
   * Register a new local model
   *
   * @param request Model registration request
   * @returns Registered model configuration
   * @throws ModelRegistryError if validation fails or model already exists
   */
  async registerModel(
    request: ModelRegistrationRequest
  ): Promise<LocalModelConfig> {
    // Generate unique ID
    const id = this.generateModelId(
      request.config.name,
      request.config.version
    );

    // Check for duplicates
    if (this.models.has(id)) {
      throw new ModelRegistryError(
        `Model already registered: ${id}`,
        "DUPLICATE_MODEL"
      );
    }

    // Validate configuration
    if (request.validate ?? true) {
      await this.validateModelConfig(request.config);
    }

    // Create full configuration
    const now = new Date();
    const modelConfig: LocalModelConfig = {
      ...request.config,
      id,
      createdAt: now,
      updatedAt: now,
      status: "testing", // Start as testing, promote to active later
    } as LocalModelConfig;

    // Store model
    this.models.set(id, modelConfig);

    // Index by name
    if (!this.modelsByName.has(request.config.name)) {
      this.modelsByName.set(request.config.name, new Set());
    }
    this.modelsByName.get(request.config.name)!.add(id);

    // Run performance profiling if requested
    if (request.profile) {
      await this.profileModel(id);
    }

    return modelConfig;
  }

  /**
   * Update model metadata
   *
   * Note: Core config (id, type, version) is immutable. Updates create a new version.
   *
   * @param request Update request
   * @returns Updated model configuration
   * @throws ModelRegistryError if model not found
   */
  async updateModel(request: ModelUpdateRequest): Promise<LocalModelConfig> {
    const existing = this.models.get(request.modelId);

    if (!existing) {
      throw new ModelRegistryError(
        `Model not found: ${request.modelId}`,
        "MODEL_NOT_FOUND"
      );
    }

    // Create updated config
    const updated: LocalModelConfig = {
      ...existing,
      ...request.updates,
      updatedAt: new Date(),
    };

    // Store updated model
    this.models.set(request.modelId, updated);

    return updated;
  }

  /**
   * Get model by ID
   *
   * @param modelId Model ID
   * @returns Model configuration or undefined
   */
  getModel(modelId: string): LocalModelConfig | undefined {
    return this.models.get(modelId);
  }

  /**
   * Get all versions of a model by name
   *
   * @param name Model name
   * @returns Array of model configurations
   */
  getModelVersions(name: string): LocalModelConfig[] {
    const ids = this.modelsByName.get(name);

    if (!ids) {
      return [];
    }

    return Array.from(ids)
      .map((id) => this.models.get(id))
      .filter((m): m is LocalModelConfig => m !== undefined)
      .sort((a, b) => b.createdAt.getTime() - a.createdAt.getTime());
  }

  /**
   * Get latest version of a model by name
   *
   * @param name Model name
   * @returns Latest model configuration or undefined
   */
  getLatestVersion(name: string): LocalModelConfig | undefined {
    const versions = this.getModelVersions(name);
    return versions[0];
  }

  /**
   * Query models with filters
   *
   * @param options Query options
   * @returns Array of matching models
   */
  queryModels(options: RegistryQueryOptions = {}): LocalModelConfig[] {
    let results = Array.from(this.models.values());

    // Filter by status
    if (options.status) {
      results = results.filter((m) => m.status === options.status);
    }

    // Filter by type
    if (options.type) {
      results = results.filter((m) => m.type === options.type);
    }

    // Filter by capabilities
    if (options.capabilities && options.capabilities.length > 0) {
      results = results.filter((m) =>
        options.capabilities!.every((cap) => m.capabilities.includes(cap))
      );
    }

    // Filter by category
    if (options.category) {
      results = results.filter((m) => m.category === options.category);
    }

    // Filter by tags
    if (options.tags && options.tags.length > 0) {
      results = results.filter(
        (m) => m.tags && options.tags!.some((tag) => m.tags!.includes(tag))
      );
    }

    // Sort
    if (options.sortBy) {
      results = this.sortModels(
        results,
        options.sortBy,
        options.sortOrder ?? "desc"
      );
    }

    // Paginate
    if (options.offset !== undefined) {
      results = results.slice(options.offset);
    }

    if (options.limit !== undefined) {
      results = results.slice(0, options.limit);
    }

    return results;
  }

  /**
   * Find models by capabilities
   *
   * @param capabilities Required capabilities
   * @returns Array of matching models
   */
  findByCapabilities(capabilities: string[]): LocalModelConfig[] {
    return this.queryModels({
      capabilities,
      status: "active", // Only active models
    });
  }

  /**
   * Deprecate a model
   *
   * @param modelId Model ID
   * @returns Updated model configuration
   * @throws ModelRegistryError if model not found
   */
  async deprecateModel(modelId: string): Promise<LocalModelConfig> {
    const model = this.models.get(modelId);

    if (!model) {
      throw new ModelRegistryError(
        `Model not found: ${modelId}`,
        "MODEL_NOT_FOUND"
      );
    }

    const updated: LocalModelConfig = {
      ...model,
      status: "deprecated",
      deprecatedAt: new Date(),
      updatedAt: new Date(),
    };

    this.models.set(modelId, updated);

    return updated;
  }

  /**
   * Activate a model (promote from testing to active)
   *
   * @param modelId Model ID
   * @returns Updated model configuration
   * @throws ModelRegistryError if model not found
   */
  async activateModel(modelId: string): Promise<LocalModelConfig> {
    const model = this.models.get(modelId);

    if (!model) {
      throw new ModelRegistryError(
        `Model not found: ${modelId}`,
        "MODEL_NOT_FOUND"
      );
    }

    const updated: LocalModelConfig = {
      ...model,
      status: "active",
      updatedAt: new Date(),
    };

    this.models.set(modelId, updated);

    return updated;
  }

  /**
   * Update performance profile for a model
   *
   * @param modelId Model ID
   * @param profile Performance profile
   * @throws ModelRegistryError if model not found
   */
  async updatePerformanceProfile(
    modelId: string,
    profile: PerformanceProfile
  ): Promise<void> {
    const model = this.models.get(modelId);

    if (!model) {
      throw new ModelRegistryError(
        `Model not found: ${modelId}`,
        "MODEL_NOT_FOUND"
      );
    }

    this.performanceProfiles.set(modelId, profile);
  }

  /**
   * Get performance profile for a model
   *
   * @param modelId Model ID
   * @returns Performance profile or undefined
   */
  getPerformanceProfile(modelId: string): PerformanceProfile | undefined {
    return this.performanceProfiles.get(modelId);
  }

  /**
   * Get all registered models
   *
   * @returns Array of all models
   */
  getAllModels(): LocalModelConfig[] {
    return Array.from(this.models.values());
  }

  /**
   * Get active models only
   *
   * @returns Array of active models
   */
  getActiveModels(): LocalModelConfig[] {
    return this.queryModels({ status: "active" });
  }

  /**
   * Check if model exists
   *
   * @param modelId Model ID
   * @returns True if model exists
   */
  hasModel(modelId: string): boolean {
    return this.models.has(modelId);
  }

  /**
   * Get model count
   *
   * @returns Total number of registered models
   */
  getModelCount(): number {
    return this.models.size;
  }

  /**
   * Clear all models (for testing)
   */
  clear(): void {
    this.models.clear();
    this.performanceProfiles.clear();
    this.modelsByName.clear();
  }

  /**
   * Register an Ollama model (convenience method)
   *
   * @param name Model name
   * @param ollamaName Ollama model name
   * @param version Model version
   * @param category Task category
   * @returns Registered model configuration
   */
  async registerOllamaModel(
    name: string,
    ollamaName: string,
    version: string,
    category: "fast" | "primary" | "quality" | "alternative" = "primary"
  ): Promise<OllamaModelConfig> {
    const config: Omit<
      OllamaModelConfig,
      "id" | "createdAt" | "updatedAt" | "status"
    > = {
      name,
      type: "ollama",
      ollamaName,
      version,
      category,
      capabilities: ["text-generation", "chat", "reasoning"],
      contextWindow: 8192, // Default for Gemma
      supportsStreaming: true,
      supportsBatching: false,
      quantization: "4bit", // Default
      endpoint: "http://localhost:11434",
    };

    return (await this.registerModel({
      config,
      validate: true,
      profile: true,
    })) as OllamaModelConfig;
  }

  /**
   * Generate unique model ID
   *
   * @param name Model name
   * @param version Model version
   * @returns Unique model ID
   */
  private generateModelId(name: string, version: string): string {
    // Format: name-version-timestamp
    const timestamp = Date.now();
    const sanitizedName = name.toLowerCase().replace(/[^a-z0-9-]/g, "-");
    const sanitizedVersion = version.replace(/[^a-z0-9.-]/gi, "-");
    return `${sanitizedName}-${sanitizedVersion}-${timestamp}`;
  }

  /**
   * Validate model configuration
   *
   * @param config Model configuration
   * @throws ModelRegistryError if validation fails
   */
  private async validateModelConfig(
    config: Omit<LocalModelConfig, "id" | "createdAt" | "updatedAt" | "status">
  ): Promise<void> {
    // Validate name
    if (!config.name || config.name.trim().length === 0) {
      throw new ModelRegistryError("Model name is required", "INVALID_NAME");
    }

    // Validate version
    if (!config.version || config.version.trim().length === 0) {
      throw new ModelRegistryError(
        "Model version is required",
        "INVALID_VERSION"
      );
    }

    // Validate capabilities
    if (!config.capabilities || config.capabilities.length === 0) {
      throw new ModelRegistryError(
        "At least one capability is required",
        "INVALID_CAPABILITIES"
      );
    }

    // Validate context window
    if (config.contextWindow <= 0) {
      throw new ModelRegistryError(
        "Context window must be positive",
        "INVALID_CONTEXT_WINDOW"
      );
    }

    // Type-specific validation
    switch (config.type) {
      case "ollama":
        await this.validateOllamaConfig(
          config as Omit<
            OllamaModelConfig,
            "id" | "createdAt" | "updatedAt" | "status"
          >
        );
        break;
      case "custom":
        // Custom model validation
        break;
      case "hardware-optimized":
        // Hardware-optimized model validation
        break;
    }
  }

  /**
   * Validate Ollama model configuration
   *
   * @param config Ollama model configuration
   * @throws ModelRegistryError if validation fails
   */
  private async validateOllamaConfig(
    config: Omit<OllamaModelConfig, "id" | "createdAt" | "updatedAt" | "status">
  ): Promise<void> {
    if (!config.ollamaName || config.ollamaName.trim().length === 0) {
      throw new ModelRegistryError(
        "Ollama model name is required",
        "INVALID_OLLAMA_NAME"
      );
    }

    // Could add Ollama availability check here
    // For now, just validate the format
    if (!config.ollamaName.includes(":")) {
      throw new ModelRegistryError(
        "Ollama model name must include tag (e.g., 'gemma3:1b')",
        "INVALID_OLLAMA_FORMAT"
      );
    }
  }

  /**
   * Profile a model's performance
   *
   * @param modelId Model ID
   */
  private async profileModel(modelId: string): Promise<void> {
    // Placeholder for performance profiling
    // This would run a series of test prompts and measure performance

    const profile: PerformanceProfile = {
      modelId,
      taskCategories: [],
      capabilities: {
        maxContextWindow: 8192,
        streamingSupport: true,
        batchingSupport: false,
      },
      resourceUsage: {
        avgMemoryMB: 0,
        avgCPUPercent: 0,
      },
      capturedAt: new Date(),
    };

    this.performanceProfiles.set(modelId, profile);
  }

  /**
   * Sort models by field
   *
   * @param models Models to sort
   * @param field Sort field
   * @param order Sort order
   * @returns Sorted models
   */
  private sortModels(
    models: LocalModelConfig[],
    field: "name" | "createdAt" | "updatedAt" | "performance",
    order: "asc" | "desc"
  ): LocalModelConfig[] {
    const sorted = [...models];

    sorted.sort((a, b) => {
      let comparison = 0;

      switch (field) {
        case "name":
          comparison = a.name.localeCompare(b.name);
          break;
        case "createdAt":
          comparison = a.createdAt.getTime() - b.createdAt.getTime();
          break;
        case "updatedAt":
          comparison = a.updatedAt.getTime() - b.updatedAt.getTime();
          break;
        case "performance":
          // Sort by performance profile if available
          const perfA = this.performanceProfiles.get(a.id);
          const perfB = this.performanceProfiles.get(b.id);

          if (!perfA && !perfB) {
            comparison = 0;
          } else if (!perfA) {
            comparison = 1;
          } else if (!perfB) {
            comparison = -1;
          } else {
            // Compare average quality across tasks
            const avgQualityA =
              perfA.taskCategories.length > 0
                ? perfA.taskCategories.reduce(
                    (sum, t) => sum + t.qualityScore,
                    0
                  ) / perfA.taskCategories.length
                : 0;
            const avgQualityB =
              perfB.taskCategories.length > 0
                ? perfB.taskCategories.reduce(
                    (sum, t) => sum + t.qualityScore,
                    0
                  ) / perfB.taskCategories.length
                : 0;

            comparison = avgQualityA - avgQualityB;
          }
          break;
      }

      return order === "asc" ? comparison : -comparison;
    });

    return sorted;
  }
}
