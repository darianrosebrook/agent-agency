/**
 * Test Fixtures and Helper Functions
 * 
 * Purpose: Provide easy-to-use helpers for creating valid test data
 * that matches the actual V2 type system.
 * 
 * @author @darianrosebrook
 */

import { Task, TaskType } from '../../src/types/arbiter-orchestration';
import { AgentProfile, AgentCapabilities } from '../../src/types/agent-registry';
import { WorkingSpec } from '../../src/types/caws-types';

/**
 * Creates a minimal valid Task with all required fields populated.
 * 
 * @param overrides - Partial task properties to override defaults
 * @returns Complete Task object ready for testing
 * 
 * @example
 * ```typescript
 * const task = createMinimalTask({
 *   description: 'My test task',
 *   type: 'code-review',
 * });
 * ```
 */
export function createMinimalTask(overrides?: Partial<Task>): Task {
  const timestamp = Date.now();
  
  return {
    id: `test-task-${timestamp}`,
    description: 'Test task description',
    type: 'code-editing' as TaskType,
    requiredCapabilities: {
      languages: ['TypeScript'],
      taskTypes: ['code-editing'],
    },
    priority: 5,
    timeoutMs: 30000,
    budget: {
      maxFiles: 10,
      maxLoc: 500,
    },
    createdAt: new Date(),
    metadata: {},
    attempts: 0,
    maxAttempts: 3,
    ...overrides,
  };
}

/**
 * Creates a minimal valid WorkingSpec with all required fields populated.
 * 
 * @param overrides - Partial spec properties to override defaults
 * @returns Complete WorkingSpec object ready for testing
 * 
 * @example
 * ```typescript
 * const spec = createMinimalWorkingSpec({
 *   title: 'My feature',
 *   risk_tier: 2,
 * });
 * ```
 */
export function createMinimalWorkingSpec(overrides?: Partial<WorkingSpec>): WorkingSpec {
  const timestamp = Date.now();
  
  return {
    id: `TEST-${timestamp}`,
    title: 'Test working spec',
    mode: 'feature',
    risk_tier: 3,
    change_budget: {
      max_files: 10,
      max_loc: 500,
    },
    blast_radius: {
      modules: [],
      data_migration: false,
    },
    operational_rollback_slo: '5m',
    scope: {
      in: ['src/test/'],
      out: ['node_modules/'],
    },
    invariants: [],
    acceptance: [
      {
        id: 'A1',
        given: 'Test condition',
        when: 'Test action',
        then: 'Test result',
      },
    ],
    non_functional: {},
    contracts: [],
    ...overrides,
  };
}

/**
 * Creates a test agent profile with sensible defaults.
 * 
 * @param overrides - Partial agent properties to override defaults
 * @returns Complete AgentProfile object ready for testing
 * 
 * @example
 * ```typescript
 * const agent = createTestAgent({
 *   id: 'my-test-agent',
 *   modelFamily: 'claude-3',
 * });
 * ```
 */
export function createTestAgent(overrides?: Partial<AgentProfile>): AgentProfile {
  const timestamp = Date.now();
  const id = overrides?.id || `test-agent-${timestamp}`;
  
  return {
    id,
    name: `Test Agent ${id}`,
    modelFamily: 'gpt-4',
    capabilities: {
      taskTypes: ['code-editing', 'code-review'],
      languages: ['TypeScript', 'JavaScript'],
      specializations: ['AST analysis', 'Performance optimization'],
    },
    performanceHistory: {
      successRate: 1.0,
      averageQuality: 0.85,
      averageLatency: 150,
      taskCount: 0,
    },
    currentLoad: {
      activeTasks: 0,
      queuedTasks: 0,
      utilizationPercent: 0,
    },
    registeredAt: new Date().toISOString(),
    lastActiveAt: new Date().toISOString(),
    ...overrides,
  };
}

/**
 * Creates a test agent with specific capabilities.
 * 
 * @param capabilities - Agent capabilities to set
 * @param overrides - Additional agent properties to override
 * @returns AgentProfile with specified capabilities
 * 
 * @example
 * ```typescript
 * const pythonAgent = createAgentWithCapabilities({
 *   languages: ['Python'],
 *   taskTypes: ['data-analysis'],
 * });
 * ```
 */
export function createAgentWithCapabilities(
  capabilities: Partial<AgentCapabilities>,
  overrides?: Partial<AgentProfile>
): AgentProfile {
  return createTestAgent({
    ...overrides,
    capabilities: {
      taskTypes: (capabilities.taskTypes || ['code-editing']) as any,
      languages: (capabilities.languages || ['TypeScript']) as any,
      specializations: (capabilities.specializations || []) as any,
    },
  });
}

/**
 * Creates multiple test agents with incrementing IDs.
 * 
 * @param count - Number of agents to create
 * @param baseOverrides - Base properties to apply to all agents
 * @returns Array of AgentProfile objects
 * 
 * @example
 * ```typescript
 * const agents = createMultipleAgents(5, {
 *   modelFamily: 'gpt-4',
 * });
 * ```
 */
export function createMultipleAgents(
  count: number,
  baseOverrides?: Partial<AgentProfile>
): AgentProfile[] {
  return Array.from({ length: count }, (_, i) =>
    createTestAgent({
      id: `test-agent-${i}`,
      name: `Test Agent ${i}`,
      ...baseOverrides,
    })
  );
}

/**
 * Creates a test task that requires specific capabilities.
 * 
 * @param capabilities - Required capabilities for the task
 * @param overrides - Additional task properties to override
 * @returns Task with specified capability requirements
 * 
 * @example
 * ```typescript
 * const task = createTaskRequiring({
 *   languages: ['Python'],
 *   taskTypes: ['data-analysis'],
 * });
 * ```
 */
export function createTaskRequiring(
  capabilities: {
    languages?: string[];
    taskTypes?: string[];
    specializations?: string[];
  },
  overrides?: Partial<Task>
): Task {
  return createMinimalTask({
    ...overrides,
    requiredCapabilities: {
      languages: capabilities.languages as any,
      taskTypes: capabilities.taskTypes as any,
      specializations: capabilities.specializations as any,
    },
  });
}

/**
 * Creates a working spec for a specific risk tier.
 * 
 * @param riskTier - Risk tier (1=critical, 2=standard, 3=low)
 * @param overrides - Additional spec properties to override
 * @returns WorkingSpec with appropriate risk tier configuration
 * 
 * @example
 * ```typescript
 * const criticalSpec = createSpecForRiskTier(1, {
 *   title: 'Critical security fix',
 * });
 * ```
 */
export function createSpecForRiskTier(
  riskTier: 1 | 2 | 3,
  overrides?: Partial<WorkingSpec>
): WorkingSpec {
  const baseSpec = createMinimalWorkingSpec({ ...overrides, risk_tier: riskTier });
  
  // Tier 1 and 2 require contracts
  if (riskTier === 1 || riskTier === 2) {
    baseSpec.contracts = overrides?.contracts || [
      {
        type: 'typescript',
        path: 'src/types/test.ts',
      } as any,
    ];
  }
  
  return baseSpec;
}

/**
 * Creates an invalid working spec for testing validation.
 * 
 * @param invalidationType - Type of validation failure to create
 * @returns Partial WorkingSpec that should fail validation
 * 
 * @example
 * ```typescript
 * const invalidSpec = createInvalidSpec('missing-acceptance');
 * ```
 */
export function createInvalidSpec(
  invalidationType: 'missing-acceptance' | 'empty-scope' | 'missing-contracts'
): Partial<WorkingSpec> {
  const baseSpec = createMinimalWorkingSpec() as any;
  
  switch (invalidationType) {
    case 'missing-acceptance':
      baseSpec.acceptance = [];
      break;
    case 'empty-scope':
      baseSpec.scope = { in: [], out: [] };
      break;
    case 'missing-contracts':
      baseSpec.risk_tier = 2;
      baseSpec.contracts = [];
      break;
  }
  
  return baseSpec;
}

/**
 * Creates a batch of tasks for load testing.
 * 
 * @param count - Number of tasks to create
 * @param baseOverrides - Base properties to apply to all tasks
 * @returns Array of Task objects
 * 
 * @example
 * ```typescript
 * const tasks = createTaskBatch(100, {
 *   type: 'code-review',
 *   priority: 5,
 * });
 * ```
 */
export function createTaskBatch(
  count: number,
  baseOverrides?: Partial<Task>
): Task[] {
  return Array.from({ length: count }, (_, i) =>
    createMinimalTask({
      id: `batch-task-${i}`,
      description: `Batch task ${i}`,
      ...baseOverrides,
    })
  );
}

/**
 * Delays execution for testing async operations.
 * 
 * @param ms - Milliseconds to delay
 * @returns Promise that resolves after delay
 * 
 * @example
 * ```typescript
 * await delay(100); // Wait 100ms
 * ```
 */
export function delay(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * Creates a mock performance outcome for testing.
 * 
 * @param success - Whether the task succeeded
 * @param overrides - Additional outcome properties
 * @returns Performance outcome object
 * 
 * @example
 * ```typescript
 * const outcome = createMockOutcome(true, { qualityScore: 0.9 });
 * ```
 */
export function createMockOutcome(
  success: boolean,
  overrides?: {
    latencyMs?: number;
    qualityScore?: number;
    tokensUsed?: number;
    error?: string;
  }
) {
  return {
    success,
    latencyMs: overrides?.latencyMs ?? 150,
    qualityScore: success ? (overrides?.qualityScore ?? 0.85) : 0,
    tokensUsed: overrides?.tokensUsed,
    error: success ? undefined : (overrides?.error ?? 'Task failed'),
  };
}

/**
 * Generates a unique test ID.
 * 
 * @param prefix - Prefix for the ID
 * @returns Unique ID string
 * 
 * @example
 * ```typescript
 * const id = generateTestId('agent'); // 'agent-1697123456789'
 * ```
 */
export function generateTestId(prefix: string = 'test'): string {
  return `${prefix}-${Date.now()}-${Math.random().toString(36).substring(7)}`;
}

/**
 * Type guard to check if a spec is valid.
 * 
 * @param spec - Spec to validate
 * @returns True if spec has all required fields
 */
export function isValidSpec(spec: any): spec is WorkingSpec {
  return !!(
    spec.id &&
    spec.title &&
    spec.mode &&
    spec.risk_tier &&
    spec.blast_radius &&
    spec.operational_rollback_slo &&
    spec.scope &&
    spec.invariants &&
    spec.acceptance &&
    spec.non_functional &&
    spec.contracts
  );
}

/**
 * Type guard to check if a task is valid.
 * 
 * @param task - Task to validate
 * @returns True if task has all required fields
 */
export function isValidTask(task: any): task is Task {
  return !!(
    task.id &&
    task.description &&
    task.type &&
    task.requiredCapabilities &&
    typeof task.priority === 'number' &&
    typeof task.timeoutMs === 'number' &&
    task.budget &&
    task.createdAt &&
    task.metadata !== undefined &&
    typeof task.attempts === 'number' &&
    typeof task.maxAttempts === 'number'
  );
}

