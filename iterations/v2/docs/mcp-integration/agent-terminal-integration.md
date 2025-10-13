# Agent Terminal Integration Guide

**Component**: INFRA-005 - MCP Terminal Access Layer Integration
**Status**: Ready for Implementation
**Risk Tier**: 1 (Critical - Command Execution)

---

## Overview

This guide shows how to integrate task runner agents with the MCP Terminal Access Layer to enable secure command execution. Agents can now run tests, install dependencies, build projects, and perform other development tasks through isolated terminal sessions.

### Key Integration Points

- **MCP Protocol**: Agents communicate via Model Context Protocol
- **Session Management**: Each task gets an isolated terminal session
- **Security Controls**: All commands validated against allowlist
- **Resource Limits**: Automatic cleanup and timeout enforcement

---

## Agent Architecture Integration

### Current Agent Flow

```
Task Assigned → Agent Processing → Task Completion
```

### Updated Agent Flow with Terminal Access

```
Task Assigned → Agent Processing → Terminal Session Creation → Command Execution → Session Cleanup → Task Completion
```

### Integration Points

1. **Task Analysis**: Determine if task requires terminal commands
2. **Session Management**: Create/cleanup terminal sessions per task
3. **Command Execution**: Use MCP tools for secure command execution
4. **Error Handling**: Handle terminal-specific errors and timeouts
5. **Resource Cleanup**: Ensure sessions are always closed

---

## Integration Examples

### 1. Node.js Project Testing Agent

```typescript
class NodeTestAgent {
  private mcpClient: MCPClient;

  async executeTask(task: Task): Promise<TaskResult> {
    // Check if task requires terminal access
    if (!this.requiresTerminalAccess(task)) {
      return this.executeWithoutTerminal(task);
    }

    // Create terminal session for this task
    const sessionResult = await this.mcpClient.callTool(
      "terminal_create_session",
      {
        taskId: task.id,
        agentId: this.agentId,
        workingDirectory: task.metadata?.workingDirectory || "./",
      }
    );

    if (!sessionResult.success) {
      throw new Error(
        `Failed to create terminal session: ${sessionResult.error}`
      );
    }

    const sessionId = sessionResult.sessionId;

    try {
      // Execute test commands in session
      const results = await this.runTestSuite(sessionId, task);

      return {
        success: results.allPassed,
        output: results.summary,
        metadata: { testResults: results.details },
      };
    } catch (error) {
      // Log terminal execution errors
      console.error(`Terminal execution failed for task ${task.id}:`, error);
      throw error;
    } finally {
      // Always cleanup session
      await this.mcpClient.callTool("terminal_close_session", {
        sessionId,
      });
    }
  }

  private async runTestSuite(
    sessionId: string,
    task: Task
  ): Promise<TestResults> {
    const results: TestCommandResult[] = [];

    // Install dependencies if needed
    if (task.metadata?.installDeps) {
      const installResult = await this.mcpClient.callTool(
        "terminal_execute_command",
        {
          sessionId,
          command: "npm",
          args: ["ci"],
        }
      );
      results.push({ command: "npm ci", ...installResult });
    }

    // Run linting
    const lintResult = await this.mcpClient.callTool(
      "terminal_execute_command",
      {
        sessionId,
        command: "npm",
        args: ["run", "lint"],
      }
    );
    results.push({ command: "npm run lint", ...lintResult });

    // Run tests with coverage
    const testResult = await this.mcpClient.callTool(
      "terminal_execute_command",
      {
        sessionId,
        command: "npm",
        args: ["test", "--coverage"],
        timeout: 300000, // 5 minutes for tests
      }
    );
    results.push({ command: "npm test", ...testResult });

    return this.analyzeResults(results);
  }

  private requiresTerminalAccess(task: Task): boolean {
    // Check task type or metadata for terminal requirements
    return (
      task.type === "test_execution" ||
      task.type === "build" ||
      task.type === "deployment" ||
      task.metadata?.requiresTerminal === true
    );
  }
}
```

### 2. Python Package Agent

```typescript
class PythonPackageAgent {
  async executeTask(task: Task): Promise<TaskResult> {
    const session = await this.mcpClient.callTool("terminal_create_session", {
      taskId: task.id,
      agentId: this.agentId,
      workingDirectory: task.metadata?.projectPath || "./",
    });

    try {
      // Install dependencies
      await this.safeExecute(session.sessionId, "pip", [
        "install",
        "-r",
        "requirements.txt",
      ]);

      // Run tests
      await this.safeExecute(session.sessionId, "python", [
        "-m",
        "pytest",
        "--cov=src",
      ]);

      // Build package
      await this.safeExecute(session.sessionId, "python", [
        "setup.py",
        "sdist",
        "bdist_wheel",
      ]);

      return { success: true, message: "Python package built successfully" };
    } finally {
      await this.mcpClient.callTool("terminal_close_session", {
        sessionId: session.sessionId,
      });
    }
  }

  private async safeExecute(
    sessionId: string,
    command: string,
    args: string[]
  ): Promise<any> {
    const result = await this.mcpClient.callTool("terminal_execute_command", {
      sessionId,
      command,
      args,
      timeout: 120000, // 2 minutes
    });

    if (!result.success) {
      throw new Error(
        `Command failed: ${command} ${args.join(" ")} - ${result.stderr}`
      );
    }

    return result;
  }
}
```

### 3. Docker Build Agent

```typescript
class DockerBuildAgent {
  async executeTask(task: Task): Promise<TaskResult> {
    const session = await this.mcpClient.callTool("terminal_create_session", {
      taskId: task.id,
      agentId: this.agentId,
      workingDirectory: task.metadata?.buildContext || "./",
    });

    try {
      const imageName = task.metadata?.imageName || "myapp:latest";

      // Build Docker image
      const buildResult = await this.mcpClient.callTool(
        "terminal_execute_command",
        {
          sessionId: session.sessionId,
          command: "docker",
          args: ["build", "-t", imageName, "."],
          timeout: 600000, // 10 minutes for builds
        }
      );

      if (!buildResult.success) {
        throw new Error(`Docker build failed: ${buildResult.stderr}`);
      }

      // Run tests in container if specified
      if (task.metadata?.runTests) {
        await this.mcpClient.callTool("terminal_execute_command", {
          sessionId: session.sessionId,
          command: "docker",
          args: ["run", "--rm", imageName, "npm", "test"],
        });
      }

      return {
        success: true,
        message: `Docker image ${imageName} built successfully`,
        metadata: { imageName, buildLog: buildResult.stdout },
      };
    } finally {
      await this.mcpClient.callTool("terminal_close_session", {
        sessionId: session.sessionId,
      });
    }
  }
}
```

---

## Error Handling Patterns

### Terminal-Specific Errors

```typescript
async executeCommandSafely(sessionId: string, command: string, args: string[]): Promise<any> {
  try {
    const result = await this.mcpClient.callTool('terminal_execute_command', {
      sessionId,
      command,
      args
    });

    return result;

  } catch (error) {
    // Handle MCP communication errors
    if (error.code === 'MCP_TIMEOUT') {
      throw new Error(`MCP communication timeout for command: ${command}`);
    }

    // Handle terminal tool errors
    if (error.type === 'terminal_error') {
      switch (error.error) {
        case 'COMMAND_NOT_ALLOWED':
          throw new Error(`Command not allowed: ${command}`);
        case 'UNSAFE_ARGUMENTS':
          throw new Error(`Unsafe arguments detected in: ${args.join(' ')}`);
        case 'TIMEOUT_EXCEEDED':
          throw new Error(`Command timed out: ${command}`);
        case 'SESSION_NOT_FOUND':
          throw new Error(`Terminal session not found: ${sessionId}`);
        default:
          throw new Error(`Terminal execution failed: ${error.message}`);
      }
    }

    throw error;
  }
}
```

### Session Lifecycle Management

```typescript
class TerminalSessionManager {
  private sessions = new Map<string, string>(); // taskId -> sessionId

  async getOrCreateSession(taskId: string): Promise<string> {
    if (this.sessions.has(taskId)) {
      return this.sessions.get(taskId)!;
    }

    const session = await this.mcpClient.callTool("terminal_create_session", {
      taskId,
      agentId: this.agentId,
    });

    this.sessions.set(taskId, session.sessionId);
    return session.sessionId;
  }

  async cleanupTaskSession(taskId: string): Promise<void> {
    const sessionId = this.sessions.get(taskId);
    if (sessionId) {
      await this.mcpClient.callTool("terminal_close_session", { sessionId });
      this.sessions.delete(taskId);
    }
  }

  async cleanupAllSessions(): Promise<void> {
    const cleanupPromises = Array.from(this.sessions.entries()).map(
      ([taskId, sessionId]) =>
        this.mcpClient
          .callTool("terminal_close_session", { sessionId })
          .catch((error) =>
            console.warn(`Failed to cleanup session ${sessionId}:`, error)
          )
    );

    await Promise.all(cleanupPromises);
    this.sessions.clear();
  }
}
```

---

## Task Type Integration

### Define Task Types That Require Terminal Access

```typescript
enum TaskType {
  // Tasks requiring terminal access
  TEST_EXECUTION = "test_execution",
  BUILD = "build",
  DEPLOYMENT = "deployment",
  PACKAGE_MANAGEMENT = "package_management",
  INFRASTRUCTURE = "infrastructure",

  // Tasks that may not need terminal access
  ANALYSIS = "analysis",
  REVIEW = "review",
  PLANNING = "planning",
}

// Task metadata interface
interface TaskMetadata {
  requiresTerminal?: boolean;
  workingDirectory?: string;
  timeout?: number;
  allowedCommands?: string[];
  environment?: Record<string, string>;
}
```

### Agent Registration with Capabilities

```typescript
const agentCapabilities = {
  supportedTaskTypes: [
    TaskType.TEST_EXECUTION,
    TaskType.BUILD,
    TaskType.PACKAGE_MANAGEMENT,
  ],
  terminalAccess: {
    enabled: true,
    maxConcurrentSessions: 3,
    supportedCommands: ["npm", "yarn", "git", "docker", "python", "pytest"],
  },
};
```

---

## Monitoring and Observability

### Session Metrics

```typescript
class AgentTerminalMonitor {
  private metrics = {
    sessionsCreated: 0,
    sessionsClosed: 0,
    commandsExecuted: 0,
    commandsFailed: 0,
    averageExecutionTime: 0,
  };

  recordSessionCreated(taskId: string, agentId: string): void {
    this.metrics.sessionsCreated++;
    console.log(`Terminal session created: task=${taskId}, agent=${agentId}`);
  }

  recordCommandExecuted(
    command: string,
    duration: number,
    success: boolean
  ): void {
    this.metrics.commandsExecuted++;
    if (!success) this.metrics.commandsFailed++;

    // Update average execution time
    const totalTime =
      this.metrics.averageExecutionTime * (this.metrics.commandsExecuted - 1);
    this.metrics.averageExecutionTime =
      (totalTime + duration) / this.metrics.commandsExecuted;

    console.log(
      `Command executed: ${command}, duration=${duration}ms, success=${success}`
    );
  }

  getMetrics(): typeof this.metrics {
    return { ...this.metrics };
  }
}
```

### Error Tracking

```typescript
class AgentErrorTracker {
  private errors = new Map<string, number>();

  recordError(errorType: string, context?: any): void {
    const count = this.errors.get(errorType) || 0;
    this.errors.set(errorType, count + 1);

    console.error(`Terminal error: ${errorType}`, context);
  }

  getErrorCounts(): Record<string, number> {
    return Object.fromEntries(this.errors);
  }

  getTopErrors(limit = 5): Array<[string, number]> {
    return Array.from(this.errors.entries())
      .sort((a, b) => b[1] - a[1])
      .slice(0, limit);
  }
}
```

---

## Best Practices

### Session Management

1. **One Session Per Task**: Create a new session for each task
2. **Always Cleanup**: Use try/finally blocks to ensure session closure
3. **Resource Limits**: Respect timeout and output size limits
4. **Error Isolation**: Session failures shouldn't affect other tasks

### Command Execution

1. **Validate Commands**: Check if command is needed before executing
2. **Safe Arguments**: Avoid shell metacharacters and variable expansion
3. **Timeout Planning**: Set appropriate timeouts for long-running commands
4. **Output Handling**: Be prepared for large output and truncation

### Error Handling

1. **Graceful Degradation**: Continue with alternative approaches if terminal fails
2. **Detailed Logging**: Log all terminal operations for debugging
3. **User Feedback**: Provide clear error messages about terminal issues
4. **Retry Logic**: Implement retry for transient terminal errors

### Security

1. **Input Validation**: Never pass untrusted input to commands
2. **Command Allowlist**: Only use pre-approved commands
3. **Environment Sanitization**: Filter sensitive environment variables
4. **Audit Logging**: Track all terminal operations

---

## Migration Guide

### For Existing Agents

1. **Add Terminal Detection**: Check if tasks require terminal access
2. **Update Task Execution**: Integrate session creation/cleanup
3. **Add Error Handling**: Handle terminal-specific errors
4. **Update Capabilities**: Register terminal access capabilities

### Example Migration

```typescript
// Before: No terminal access
class OldAgent {
  async executeTask(task: Task): Promise<TaskResult> {
    // Process task without terminal
    return { success: true };
  }
}

// After: With terminal access
class NewAgent {
  async executeTask(task: Task): Promise<TaskResult> {
    if (!this.requiresTerminal(task)) {
      return this.executeWithoutTerminal(task);
    }

    const session = await this.mcpClient.callTool("terminal_create_session", {
      taskId: task.id,
      agentId: this.agentId,
    });

    try {
      return await this.executeWithTerminal(task, session.sessionId);
    } finally {
      await this.mcpClient.callTool("terminal_close_session", {
        sessionId: session.sessionId,
      });
    }
  }
}
```

---

## Testing Integration

### Unit Tests

```typescript
describe("Agent Terminal Integration", () => {
  let mockMcpClient: jest.Mocked<MCPClient>;

  beforeEach(() => {
    mockMcpClient = {
      callTool: jest.fn(),
    } as any;
  });

  it("should create and cleanup terminal session", async () => {
    mockMcpClient.callTool
      .mockResolvedValueOnce({ sessionId: "session-123" }) // create
      .mockResolvedValueOnce({ success: true }) // execute
      .mockResolvedValueOnce({ success: true }); // close

    const agent = new NodeTestAgent(mockMcpClient);
    const result = await agent.executeTask(testTask);

    expect(result.success).toBe(true);
    expect(mockMcpClient.callTool).toHaveBeenCalledWith(
      "terminal_create_session",
      expect.any(Object)
    );
    expect(mockMcpClient.callTool).toHaveBeenCalledWith(
      "terminal_close_session",
      { sessionId: "session-123" }
    );
  });
});
```

### Integration Tests

```typescript
describe("Terminal Integration E2E", () => {
  it("should execute real commands in isolated sessions", async () => {
    // This would run against a real MCP server with terminal access
    const agent = new NodeTestAgent(realMcpClient);
    const result = await agent.executeTask({
      id: "test-task",
      type: "test_execution",
      metadata: { workingDirectory: "/tmp/test-project" },
    });

    expect(result.success).toBe(true);
    expect(result.metadata.testResults).toBeDefined();
  });
});
```

---

## Performance Considerations

### Session Overhead

- **Creation**: ~50-100ms per session
- **Command Execution**: ~200-500ms baseline + command time
- **Cleanup**: ~25-50ms per session

### Resource Limits

```typescript
const AGENT_LIMITS = {
  maxConcurrentSessions: 5, // Per agent
  maxSessionLifetime: 1800000, // 30 minutes
  maxCommandsPerSession: 50, // Commands per session
  maxOutputSize: 1024 * 1024, // 1MB per command
};
```

### Monitoring Alerts

Set up alerts for:

- High session creation rate (>10/minute per agent)
- Command failure rate (>20%)
- Session cleanup failures
- Timeout rate (>10%)

---

## Troubleshooting

### Common Issues

#### Session Creation Fails

**Problem**: `terminal_create_session` returns error
**Solution**:

- Check MCP server is running with terminal access enabled
- Verify `ENABLE_TERMINAL_ACCESS=true` environment variable
- Check agent permissions

#### Command Not Allowed

**Problem**: `COMMAND_NOT_ALLOWED` error
**Solution**:

- Check command is in `apps/tools/caws/tools-allow.json`
- Use alternative allowlisted command
- Request addition to allowlist if legitimate

#### Session Not Found

**Problem**: Commands fail with `SESSION_NOT_FOUND`
**Solution**:

- Ensure session is created before command execution
- Check session ID is correct
- Verify session wasn't prematurely closed

#### Timeout Errors

**Problem**: Commands timeout unexpectedly
**Solution**:

- Increase timeout parameter
- Optimize command performance
- Break long commands into smaller steps

---

## Related Documentation

- [Terminal Access User Guide](../terminal-access.md)
- [MCP Protocol Specification](../MCP/README.md)
- [Agent Development Guide](../agents/full-guide.md)
- [Security Best Practices](../security/README.md)

---

## Support

**Component Owner**: @darianrosebrook
**Integration Support**: MCP Terminal Access team
**Security Contact**: Security team for allowlist changes
**Documentation Updated**: 2025-10-13

For integration issues, check the [troubleshooting guide](#troubleshooting) or create an issue with the `terminal-integration` label.
