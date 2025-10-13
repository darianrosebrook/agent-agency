/**
 * Agent Terminal Integration Example
 *
 * Complete example showing how to integrate an existing task runner agent
 * with the MCP Terminal Access Layer for secure command execution.
 *
 * @author @darianrosebrook
 */

import { MCPClient } from './mcp-client'; // Assume MCP client implementation
import type { Task, TaskResult } from '../src/types/task-types';

/**
 * BEFORE: Agent without terminal access
 * - Limited to analysis and planning tasks
 * - Cannot execute tests, builds, or deployments
 * - No access to file system operations
 */
class BasicAgent {
  private agentId = 'basic-agent-001';

  async executeTask(task: Task): Promise<TaskResult> {
    switch (task.type) {
      case 'analysis':
        return this.analyzeCode(task);
      case 'planning':
        return this.createPlan(task);
      default:
        throw new Error(`Unsupported task type: ${task.type}`);
    }
  }

  private async analyzeCode(task: Task): Promise<TaskResult> {
    // Can only analyze code in memory
    const analysis = {
      complexity: 'medium',
      issues: ['Missing error handling', 'Could use optimization'],
      recommendations: ['Add try/catch blocks', 'Consider caching']
    };

    return {
      success: true,
      output: JSON.stringify(analysis, null, 2),
      metadata: { analysisType: 'static' }
    };
  }

  private async createPlan(task: Task): Promise<TaskResult> {
    const plan = {
      steps: [
        'Analyze requirements',
        'Design solution',
        'Implement features',
        'Test implementation',
        'Deploy to production'
      ],
      estimatedHours: 40,
      riskLevel: 'medium'
    };

    return {
      success: true,
      output: JSON.stringify(plan, null, 2),
      metadata: { planType: 'development' }
    };
  }
}

/**
 * AFTER: Agent with terminal access
 * - Can execute tests, builds, deployments
 * - Full access to development workflow
 * - Secure command execution with audit trail
 */
class EnhancedAgent {
  private agentId = 'enhanced-agent-001';
  private mcpClient: MCPClient;
  private terminalSessions = new Map<string, string>(); // taskId -> sessionId

  constructor(mcpClient: MCPClient) {
    this.mcpClient = mcpClient;
  }

  async executeTask(task: Task): Promise<TaskResult> {
    // Determine if task requires terminal access
    if (this.requiresTerminalAccess(task)) {
      return this.executeWithTerminal(task);
    } else {
      return this.executeWithoutTerminal(task);
    }
  }

  private requiresTerminalAccess(task: Task): boolean {
    // Check task type
    const terminalTaskTypes = [
      'test_execution',
      'build',
      'deployment',
      'package_management',
      'infrastructure'
    ];

    if (terminalTaskTypes.includes(task.type)) {
      return true;
    }

    // Check metadata
    return task.metadata?.requiresTerminal === true ||
           task.metadata?.workingDirectory !== undefined ||
           task.metadata?.installDeps === true ||
           task.metadata?.runTests === true;
  }

  private async executeWithTerminal(task: Task): Promise<TaskResult> {
    let sessionId: string;

    try {
      // Create terminal session
      sessionId = await this.createTerminalSession(task);

      // Execute task-specific commands
      switch (task.type) {
        case 'test_execution':
          return await this.executeTestSuite(task, sessionId);
        case 'build':
          return await this.executeBuild(task, sessionId);
        case 'deployment':
          return await this.executeDeployment(task, sessionId);
        case 'package_management':
          return await this.executePackageManagement(task, sessionId);
        default:
          // Handle custom terminal tasks
          return await this.executeCustomTerminalTask(task, sessionId);
      }

    } catch (error) {
      console.error(`Terminal execution failed for task ${task.id}:`, error);
      throw error;

    } finally {
      // Always cleanup session
      if (sessionId) {
        await this.cleanupTerminalSession(task.id, sessionId);
      }
    }
  }

  private async createTerminalSession(task: Task): Promise<string> {
    const sessionResult = await this.mcpClient.callTool('terminal_create_session', {
      taskId: task.id,
      agentId: this.agentId,
      workingDirectory: task.metadata?.workingDirectory || './',
      environment: task.metadata?.environment || {}
    });

    if (!sessionResult.success) {
      throw new Error(`Failed to create terminal session: ${sessionResult.error}`);
    }

    const sessionId = sessionResult.sessionId;
    this.terminalSessions.set(task.id, sessionId);

    console.log(`Created terminal session ${sessionId} for task ${task.id}`);
    return sessionId;
  }

  private async cleanupTerminalSession(taskId: string, sessionId: string): Promise<void> {
    try {
      await this.mcpClient.callTool('terminal_close_session', { sessionId });
      this.terminalSessions.delete(taskId);
      console.log(`Cleaned up terminal session ${sessionId} for task ${taskId}`);
    } catch (error) {
      console.warn(`Failed to cleanup terminal session ${sessionId}:`, error);
    }
  }

  private async executeTestSuite(task: Task, sessionId: string): Promise<TaskResult> {
    const results: any[] = [];

    // Install dependencies if requested
    if (task.metadata?.installDeps) {
      const installResult = await this.safeExecuteCommand(sessionId, 'npm', ['ci']);
      results.push({ stage: 'install', ...installResult });
    }

    // Run linting
    if (task.metadata?.runLint !== false) { // Default true
      const lintResult = await this.safeExecuteCommand(sessionId, 'npm', ['run', 'lint']);
      results.push({ stage: 'lint', ...lintResult });
    }

    // Run tests
    const testArgs = task.metadata?.testArgs || ['test'];
    if (task.metadata?.coverage) {
      testArgs.push('--coverage');
    }

    const testResult = await this.safeExecuteCommand(sessionId, 'npm', testArgs, 300000); // 5 min timeout
    results.push({ stage: 'test', ...testResult });

    // Generate summary
    const passed = results.every(r => r.success);
    const summary = results.map(r => `${r.stage}: ${r.success ? 'PASS' : 'FAIL'}`).join(', ');

    return {
      success: passed,
      output: summary,
      metadata: {
        testResults: results,
        totalStages: results.length,
        passedStages: results.filter(r => r.success).length
      }
    };
  }

  private async executeBuild(task: Task, sessionId: string): Promise<TaskResult> {
    const buildArgs = task.metadata?.buildArgs || ['run', 'build'];
    const timeout = task.metadata?.buildTimeout || 300000; // 5 minutes

    const buildResult = await this.safeExecuteCommand(sessionId, 'npm', buildArgs, timeout);

    return {
      success: buildResult.success,
      output: buildResult.success ? 'Build completed successfully' : `Build failed: ${buildResult.stderr}`,
      metadata: {
        buildTime: buildResult.duration,
        buildLog: buildResult.stdout
      }
    };
  }

  private async executeDeployment(task: Task, sessionId: string): Promise<TaskResult> {
    const steps: any[] = [];

    // Pre-deployment checks
    if (task.metadata?.healthCheck) {
      const healthResult = await this.safeExecuteCommand(sessionId, 'npm', ['run', 'health-check']);
      steps.push({ step: 'health-check', ...healthResult });
    }

    // Build if not already built
    if (task.metadata?.buildBeforeDeploy) {
      const buildResult = await this.safeExecuteCommand(sessionId, 'npm', ['run', 'build']);
      steps.push({ step: 'build', ...buildResult });
    }

    // Run deployment
    const deployCommand = task.metadata?.deployCommand || ['run', 'deploy'];
    const deployResult = await this.safeExecuteCommand(sessionId, 'npm', deployCommand, 600000); // 10 min
    steps.push({ step: 'deploy', ...deployResult });

    // Post-deployment verification
    if (task.metadata?.postDeployCheck) {
      const verifyResult = await this.safeExecuteCommand(sessionId, 'npm', ['run', 'post-deploy-check']);
      steps.push({ step: 'verify', ...verifyResult });
    }

    const success = steps.every(s => s.success);
    const summary = steps.map(s => `${s.step}: ${s.success ? 'OK' : 'FAILED'}`).join(' → ');

    return {
      success,
      output: summary,
      metadata: { deploymentSteps: steps }
    };
  }

  private async executePackageManagement(task: Task, sessionId: string): Promise<TaskResult> {
    const packageManager = task.metadata?.packageManager || 'npm';
    const command = task.metadata?.packageCommand || 'install';

    let args: string[];
    if (packageManager === 'npm' || packageManager === 'yarn') {
      args = [command];
      if (task.metadata?.packages) {
        args.push(...task.metadata.packages);
      }
    } else if (packageManager === 'pip') {
      args = [command];
      if (task.metadata?.packages) {
        args.push(...task.metadata.packages);
      }
    } else {
      throw new Error(`Unsupported package manager: ${packageManager}`);
    }

    const result = await this.safeExecuteCommand(sessionId, packageManager, args);

    return {
      success: result.success,
      output: result.success ? 'Package management completed' : `Package management failed: ${result.stderr}`,
      metadata: {
        packageManager,
        command,
        packages: task.metadata?.packages
      }
    };
  }

  private async executeCustomTerminalTask(task: Task, sessionId: string): Promise<TaskResult> {
    // Handle tasks with custom terminal commands
    const commands = task.metadata?.commands || [];

    const results: any[] = [];
    for (const cmd of commands) {
      const result = await this.safeExecuteCommand(sessionId, cmd.command, cmd.args || [], cmd.timeout);
      results.push({
        command: cmd.command,
        args: cmd.args,
        ...result
      });
    }

    const success = results.every(r => r.success);
    const summary = results.map(r =>
      `${r.command} ${r.args?.join(' ') || ''}: ${r.success ? 'OK' : 'FAILED'}`
    ).join('\n');

    return {
      success,
      output: summary,
      metadata: { commandResults: results }
    };
  }

  private async safeExecuteCommand(
    sessionId: string,
    command: string,
    args: string[] = [],
    timeout?: number
  ): Promise<any> {
    try {
      const result = await this.mcpClient.callTool('terminal_execute_command', {
        sessionId,
        command,
        args,
        timeout
      });

      if (!result.success) {
        // Handle terminal-specific errors
        switch (result.error) {
          case 'COMMAND_NOT_ALLOWED':
            throw new Error(`Command not allowed: ${command}`);
          case 'UNSAFE_ARGUMENTS':
            throw new Error(`Unsafe arguments in command: ${command} ${args.join(' ')}`);
          case 'TIMEOUT_EXCEEDED':
            throw new Error(`Command timed out: ${command} ${args.join(' ')}`);
          case 'SESSION_NOT_FOUND':
            throw new Error(`Terminal session not found: ${sessionId}`);
          default:
            throw new Error(`Command failed: ${command} ${args.join(' ')} - ${result.stderr}`);
        }
      }

      return result;

    } catch (error) {
      // Log the error with context
      console.error(`Command execution error: ${command} ${args.join(' ')}`, error);
      throw error;
    }
  }

  private async executeWithoutTerminal(task: Task): Promise<TaskResult> {
    // Fallback to original logic for non-terminal tasks
    switch (task.type) {
      case 'analysis':
        return this.analyzeCode(task);
      case 'planning':
        return this.createPlan(task);
      default:
        throw new Error(`Unsupported task type: ${task.type}`);
    }
  }

  private async analyzeCode(task: Task): Promise<TaskResult> {
    // Enhanced analysis that could use terminal for additional checks
    const basicAnalysis = {
      complexity: 'medium',
      issues: ['Missing error handling', 'Could use optimization'],
      recommendations: ['Add try/catch blocks', 'Consider caching']
    };

    // Could potentially run additional analysis commands here
    // For example: running static analysis tools, checking file sizes, etc.

    return {
      success: true,
      output: JSON.stringify(basicAnalysis, null, 2),
      metadata: {
        analysisType: 'enhanced',
        terminalUsed: false
      }
    };
  }

  private async createPlan(task: Task): Promise<TaskResult> {
    // Enhanced planning that could analyze existing codebase
    const plan = {
      steps: [
        'Analyze existing codebase',
        'Identify integration points',
        'Design terminal-based workflows',
        'Implement with MCP terminal access',
        'Test terminal integration',
        'Deploy enhanced agent'
      ],
      estimatedHours: 60,
      riskLevel: 'medium',
      terminalIntegration: true
    };

    return {
      success: true,
      output: JSON.stringify(plan, null, 2),
      metadata: {
        planType: 'terminal-enhanced',
        terminalIntegration: true
      }
    };
  }

  // Cleanup method for graceful shutdown
  async shutdown(): Promise<void> {
    console.log('Shutting down enhanced agent...');

    // Cleanup all active terminal sessions
    const cleanupPromises = Array.from(this.terminalSessions.entries()).map(
      ([taskId, sessionId]) => this.cleanupTerminalSession(taskId, sessionId)
    );

    await Promise.all(cleanupPromises);
    console.log('All terminal sessions cleaned up');
  }
}

/**
 * Usage Examples
 */

async function demonstrateIntegration() {
  const mcpClient = new MCPClient({
    endpoint: 'http://localhost:3000/mcp',
    agentId: 'demo-agent'
  });

  const enhancedAgent = new EnhancedAgent(mcpClient);

  // Example 1: Test execution task
  const testTask: Task = {
    id: 'demo-test-001',
    type: 'test_execution',
    description: 'Run test suite for Node.js project',
    metadata: {
      workingDirectory: './sample-project',
      installDeps: true,
      coverage: true,
      timeout: 300000
    }
  };

  try {
    const testResult = await enhancedAgent.executeTask(testTask);
    console.log('Test execution result:', testResult);
  } catch (error) {
    console.error('Test execution failed:', error);
  }

  // Example 2: Build task
  const buildTask: Task = {
    id: 'demo-build-001',
    type: 'build',
    description: 'Build production artifacts',
    metadata: {
      workingDirectory: './sample-project',
      buildArgs: ['run', 'build:prod']
    }
  };

  try {
    const buildResult = await enhancedAgent.executeTask(buildTask);
    console.log('Build result:', buildResult);
  } catch (error) {
    console.error('Build failed:', error);
  }

  // Cleanup
  await enhancedAgent.shutdown();
}

/**
 * Task Definition Examples
 */

// Test execution task
const testTaskExample: Task = {
  id: 'test-suite-001',
  type: 'test_execution',
  description: 'Execute comprehensive test suite',
  metadata: {
    requiresTerminal: true,
    workingDirectory: './src',
    installDeps: true,
    runLint: true,
    coverage: true,
    testArgs: ['test', '--watchAll=false'],
    timeout: 600000 // 10 minutes
  }
};

// Build task
const buildTaskExample: Task = {
  id: 'build-prod-001',
  type: 'build',
  description: 'Production build with optimizations',
  metadata: {
    requiresTerminal: true,
    workingDirectory: './',
    buildArgs: ['run', 'build'],
    buildTimeout: 300000,
    environment: {
      NODE_ENV: 'production',
      CI: 'true'
    }
  }
};

// Custom terminal task
const customTaskExample: Task = {
  id: 'custom-ops-001',
  type: 'infrastructure',
  description: 'Custom infrastructure operations',
  metadata: {
    requiresTerminal: true,
    workingDirectory: './infrastructure',
    commands: [
      {
        command: 'terraform',
        args: ['init'],
        timeout: 120000
      },
      {
        command: 'terraform',
        args: ['plan'],
        timeout: 300000
      },
      {
        command: 'terraform',
        args: ['apply', '-auto-approve'],
        timeout: 600000
      }
    ]
  }
};

export {
  BasicAgent,
  EnhancedAgent,
  demonstrateIntegration,
  testTaskExample,
  buildTaskExample,
  customTaskExample
};

// Uncomment to run demonstration
// demonstrateIntegration().catch(console.error);
