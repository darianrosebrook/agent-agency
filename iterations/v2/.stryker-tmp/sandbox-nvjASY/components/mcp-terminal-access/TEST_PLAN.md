# Test Plan: MCP Terminal Access Layer

**Component**: INFRA-005  
**Risk Tier**: 1 (Critical - Command Execution)  
**Coverage Target**: 90%+ line coverage, 90%+ branch coverage  
**Mutation Score Target**: 75%+

---

## Test Strategy

### Pyramid Structure

```
        /\
       /  \
      / E2E \          1 workflow test
     /------\
    /        \
   /Integration\       3 integration tests
  /------------\
 /              \
/  Unit Tests    \     20+ unit tests
------------------
```

### Risk-Based Testing Approach

**Critical Paths** (must have 100% coverage):

- Command validation logic
- Session cleanup and resource management
- Timeout enforcement
- Output size limiting
- Security validation

**Standard Paths** (90%+ coverage):

- Session creation
- Command execution
- Status monitoring
- Event emission

---

## Unit Tests

### CommandValidator Tests

**File**: `tests/unit/security/CommandValidator.test.ts`

#### Test Cases

1. **Allowlist Validation**

   ```typescript
   describe("CommandValidator", () => {
     describe("isCommandAllowed", () => {
       it("should allow commands in tools-allow.json", () => {
         expect(validator.isCommandAllowed("npm")).toBe(true);
         expect(validator.isCommandAllowed("git")).toBe(true);
         expect(validator.isCommandAllowed("node")).toBe(true);
       });

       it("should reject commands not in allowlist", () => {
         expect(validator.isCommandAllowed("rm")).toBe(false);
         expect(validator.isCommandAllowed("sudo")).toBe(false);
         expect(validator.isCommandAllowed("eval")).toBe(false);
       });

       it("should extract base command from full path", () => {
         expect(validator.isCommandAllowed("/usr/bin/npm")).toBe(true);
         expect(validator.isCommandAllowed("./node_modules/.bin/jest")).toBe(
           true
         );
       });
     });

     describe("validateArguments", () => {
       it("should allow safe arguments", () => {
         expect(validator.validateArguments(["test", "--coverage"])).toBe(true);
         expect(validator.validateArguments(["install", "express"])).toBe(true);
       });

       it("should reject shell metacharacters", () => {
         expect(validator.validateArguments(["test;rm -rf /"])).toBe(false);
         expect(validator.validateArguments(["test`whoami`"])).toBe(false);
         expect(validator.validateArguments(["test$(echo bad)"])).toBe(false);
         expect(validator.validateArguments(["test|grep secret"])).toBe(false);
       });

       it("should reject environment variable injection", () => {
         expect(validator.validateArguments(["test$PATH"])).toBe(false);
         expect(validator.validateArguments(["test${HOME}"])).toBe(false);
       });
     });

     describe("sanitizeEnvironment", () => {
       it("should remove sensitive environment variables", () => {
         const env = {
           PATH: "/usr/bin",
           AWS_SECRET: "secret123",
           DATABASE_PASSWORD: "pass456",
           NODE_ENV: "test",
         };

         const sanitized = validator.sanitizeEnvironment(env);
         expect(sanitized.NODE_ENV).toBe("test");
         expect(sanitized.PATH).toBe("/usr/bin");
         expect(sanitized.AWS_SECRET).toBeUndefined();
         expect(sanitized.DATABASE_PASSWORD).toBeUndefined();
       });
     });
   });
   ```

2. **Edge Cases**
   - Empty command string
   - Null/undefined inputs
   - Very long command strings (>1000 chars)
   - Unicode characters in commands
   - Case sensitivity handling

**Coverage Goal**: 95%+ (critical security code)

---

### TerminalSessionManager Tests

**File**: `tests/unit/mcp-server/TerminalSessionManager.test.ts`

#### Test Cases

1. **Session Creation**

   ```typescript
   describe("TerminalSessionManager", () => {
     describe("createSession", () => {
       it("should create session with unique ID", async () => {
         const session1 = await manager.createSession("TASK-1", "agent-1");
         const session2 = await manager.createSession("TASK-2", "agent-1");

         expect(session1.id).not.toBe(session2.id);
         expect(session1.state).toBe("idle");
       });

       it("should use provided working directory", async () => {
         const session = await manager.createSession("TASK-1", "agent-1", {
           workingDirectory: "/custom/path",
         });

         expect(session.workingDirectory).toBe("/custom/path");
       });

       it("should merge environment variables", async () => {
         const session = await manager.createSession("TASK-1", "agent-1", {
           environment: { CUSTOM_VAR: "value" },
         });

         expect(session.environment.CUSTOM_VAR).toBe("value");
         expect(session.environment.CAWS_TASK_ID).toBe("TASK-1");
         expect(session.environment.CAWS_AGENT_ID).toBe("agent-1");
       });

       it("should emit session:created event", async () => {
         const eventSpy = jest.fn();
         manager.on("session:created", eventSpy);

         await manager.createSession("TASK-1", "agent-1");

         expect(eventSpy).toHaveBeenCalledWith(
           expect.objectContaining({
             taskId: "TASK-1",
             agentId: "agent-1",
           })
         );
       });
     });
   });
   ```

2. **Command Execution**

   ```typescript
   describe("executeCommand", () => {
     it("should execute allowed command and return result", async () => {
       const session = await manager.createSession("TASK-1", "agent-1");

       const result = await manager.executeCommand({
         sessionId: session.id,
         command: "echo",
         args: ["hello"],
       });

       expect(result.success).toBe(true);
       expect(result.exitCode).toBe(0);
       expect(result.stdout).toContain("hello");
     });

     it("should reject disallowed command", async () => {
       const session = await manager.createSession("TASK-1", "agent-1");

       const result = await manager.executeCommand({
         sessionId: session.id,
         command: "rm",
         args: ["-rf", "/"],
       });

       expect(result.success).toBe(false);
       expect(result.error).toBe("COMMAND_NOT_ALLOWED");
     });

     it("should enforce timeout", async () => {
       const session = await manager.createSession("TASK-1", "agent-1");

       const result = await manager.executeCommand({
         sessionId: session.id,
         command: "sleep",
         args: ["10"],
         timeout: 1000, // 1 second
       });

       expect(result.success).toBe(false);
       expect(result.stderr).toContain("timeout");
     });

     it("should truncate output at 1MB", async () => {
       const session = await manager.createSession("TASK-1", "agent-1");

       // Generate large output
       const result = await manager.executeCommand({
         sessionId: session.id,
         command: "node",
         args: ["-e", 'console.log("x".repeat(2000000))'],
       });

       expect(result.stdout.length).toBeLessThanOrEqual(1024 * 1024);
       expect(result.truncated).toBe(true);
     });

     it("should update session state during execution", async () => {
       const session = await manager.createSession("TASK-1", "agent-1");

       const promise = manager.executeCommand({
         sessionId: session.id,
         command: "sleep",
         args: ["1"],
       });

       // Check state is 'running' during execution
       const runningSession = manager.getSession(session.id);
       expect(runningSession?.state).toBe("running");

       await promise;

       // Check state after completion
       const completedSession = manager.getSession(session.id);
       expect(completedSession?.state).toBe("completed");
     });
   });
   ```

3. **Session Cleanup**

   ```typescript
   describe("closeSession", () => {
     it("should close session and remove from registry", async () => {
       const session = await manager.createSession("TASK-1", "agent-1");

       await manager.closeSession(session.id);

       expect(manager.getSession(session.id)).toBeUndefined();
     });

     it("should kill running process on close", async () => {
       const session = await manager.createSession("TASK-1", "agent-1");

       // Start long-running command
       const promise = manager.executeCommand({
         sessionId: session.id,
         command: "sleep",
         args: ["100"],
       });

       // Close session while running
       await manager.closeSession(session.id);

       // Command should fail due to killed process
       await expect(promise).rejects.toThrow();
     });

     it("should emit session:closed event", async () => {
       const session = await manager.createSession("TASK-1", "agent-1");
       const eventSpy = jest.fn();
       manager.on("session:closed", eventSpy);

       await manager.closeSession(session.id);

       expect(eventSpy).toHaveBeenCalledWith(
         expect.objectContaining({ sessionId: session.id })
       );
     });

     it("should be idempotent (closing twice is safe)", async () => {
       const session = await manager.createSession("TASK-1", "agent-1");

       await manager.closeSession(session.id);
       await expect(manager.closeSession(session.id)).resolves.not.toThrow();
     });
   });
   ```

4. **Session Monitoring**
   ```typescript
   describe("getSession and listSessions", () => {
     it("should retrieve session by ID", async () => {
       const session = await manager.createSession("TASK-1", "agent-1");

       const retrieved = manager.getSession(session.id);

       expect(retrieved).toEqual(session);
     });

     it("should list all active sessions", async () => {
       await manager.createSession("TASK-1", "agent-1");
       await manager.createSession("TASK-2", "agent-2");

       const sessions = manager.listSessions();

       expect(sessions).toHaveLength(2);
     });

     it("should enforce max concurrent session limit", async () => {
       // Create 50 sessions (the max)
       for (let i = 0; i < 50; i++) {
         await manager.createSession(`TASK-${i}`, "agent-1");
       }

       // 51st session should fail
       await expect(
         manager.createSession("TASK-51", "agent-1")
       ).rejects.toThrow("Maximum concurrent sessions exceeded");
     });
   });
   ```

**Coverage Goal**: 90%+

---

## Integration Tests

**File**: `tests/integration/terminal-execution.test.ts`

### Test Cases

1. **Real Process Execution**

   ```typescript
   describe("Terminal Integration", () => {
     it("should execute real npm command", async () => {
       const manager = new TerminalSessionManager(projectRoot);
       const session = await manager.createSession("INT-1", "test-agent");

       const result = await manager.executeCommand({
         sessionId: session.id,
         command: "npm",
         args: ["--version"],
       });

       expect(result.success).toBe(true);
       expect(result.stdout).toMatch(/^\d+\.\d+\.\d+/);

       await manager.closeSession(session.id);
     });

     it("should execute git command in working directory", async () => {
       const manager = new TerminalSessionManager(projectRoot);
       const session = await manager.createSession("INT-2", "test-agent", {
         workingDirectory: projectRoot,
       });

       const result = await manager.executeCommand({
         sessionId: session.id,
         command: "git",
         args: ["status", "--short"],
       });

       expect(result.success).toBe(true);
       expect(result.exitCode).toBe(0);

       await manager.closeSession(session.id);
     });

     it("should maintain state across multiple commands", async () => {
       const manager = new TerminalSessionManager(projectRoot);
       const tmpDir = "/tmp/terminal-test-" + Date.now();

       const session = await manager.createSession("INT-3", "test-agent", {
         workingDirectory: tmpDir,
       });

       // Create directory
       await manager.executeCommand({
         sessionId: session.id,
         command: "mkdir",
         args: ["-p", tmpDir],
       });

       // Create file
       await manager.executeCommand({
         sessionId: session.id,
         command: "touch",
         args: ["test.txt"],
       });

       // Verify file exists
       const result = await manager.executeCommand({
         sessionId: session.id,
         command: "ls",
         args: [],
       });

       expect(result.stdout).toContain("test.txt");

       await manager.closeSession(session.id);
     });
   });
   ```

2. **MCP Tool Integration**
   ```typescript
   describe("MCP Tools Integration", () => {
     it("should create session via MCP tool", async () => {
       const response = await mcpServer.handleToolCall({
         name: "terminal_create_session",
         arguments: {
           taskId: "MCP-1",
           agentId: "mcp-agent",
         },
       });

       expect(response.content[0].text).toContain('"success": true');
       const result = JSON.parse(response.content[0].text);
       expect(result.sessionId).toBeDefined();
     });

     it("should execute command via MCP tool", async () => {
       // Create session first
       const createResponse = await mcpServer.handleToolCall({
         name: "terminal_create_session",
         arguments: { taskId: "MCP-2", agentId: "mcp-agent" },
       });

       const createResult = JSON.parse(createResponse.content[0].text);

       // Execute command
       const execResponse = await mcpServer.handleToolCall({
         name: "terminal_execute_command",
         arguments: {
           sessionId: createResult.sessionId,
           command: "echo",
           args: ["test"],
         },
       });

       const execResult = JSON.parse(execResponse.content[0].text);
       expect(execResult.success).toBe(true);
       expect(execResult.stdout).toContain("test");
     });
   });
   ```

**Coverage Goal**: 85%+

---

## Security Tests

**File**: `tests/security/terminal-security.test.ts`

### Attack Vectors to Test

1. **Command Injection**
   ```typescript
   describe("Security Tests", () => {
     describe("Command Injection Prevention", () => {
       it("should block shell escape via semicolon", async () => {
         const result = await manager.executeCommand({
           sessionId: session.id,
           command: "echo",
           args: ["test; rm -rf /"],
         });

         expect(result.success).toBe(false);
         expect(result.error).toBe("COMMAND_NOT_ALLOWED");
       });

       it("should block command substitution", async () => {
         const result = await manager.executeCommand({
           sessionId: session.id,
           command: "echo",
           args: ["test$(whoami)"],
         });

         expect(result.success).toBe(false);
       });

       it("should block backtick execution", async () => {
         const result = await manager.executeCommand({
           sessionId: session.id,
           command: "echo",
           args: ["test`cat /etc/passwd`"],
         });

         expect(result.success).toBe(false);
       });
     });

     describe("Path Traversal Prevention", () => {
       it("should block directory traversal", async () => {
         const result = await manager.executeCommand({
           sessionId: session.id,
           command: "cat",
           args: ["../../../etc/passwd"],
         });

         expect(result.success).toBe(false);
       });
     });

     describe("Environment Variable Protection", () => {
       it("should not leak sensitive environment variables", async () => {
         const session = await manager.createSession("SEC-1", "agent", {
           environment: { AWS_SECRET: "secret123" },
         });

         const result = await manager.executeCommand({
           sessionId: session.id,
           command: "env",
           args: [],
         });

         // Sensitive vars should be filtered
         expect(result.stdout).not.toContain("AWS_SECRET");
       });
     });
   });
   ```

**Required**: Zero successful attack vectors

---

## E2E Tests

**File**: `tests/e2e/mcp-terminal-workflow.test.ts`

### Test Cases

1. **Complete Agent Workflow**
   ```typescript
   describe("E2E: Agent Terminal Workflow", () => {
     it("should complete full task lifecycle", async () => {
       // Agent creates session
       const createResult = await agentClient.createTerminalSession({
         taskId: "E2E-001",
         agentId: "test-worker",
       });

       expect(createResult.success).toBe(true);
       const sessionId = createResult.sessionId;

       // Agent installs dependencies
       const installResult = await agentClient.executeCommand({
         sessionId,
         command: "npm",
         args: ["install"],
       });

       expect(installResult.success).toBe(true);

       // Agent runs tests
       const testResult = await agentClient.executeCommand({
         sessionId,
         command: "npm",
         args: ["test"],
       });

       expect(testResult.success).toBe(true);
       expect(testResult.exitCode).toBe(0);

       // Agent checks status
       const statusResult = await agentClient.getSessionStatus(sessionId);
       expect(statusResult.session.state).toBe("completed");
       expect(statusResult.session.commandCount).toBe(2);

       // Agent closes session
       const closeResult = await agentClient.closeSession(sessionId);
       expect(closeResult.success).toBe(true);

       // Verify session no longer exists
       const finalStatus = await agentClient.getSessionStatus(sessionId);
       expect(finalStatus.success).toBe(false);
       expect(finalStatus.error).toBe("SESSION_NOT_FOUND");
     });

     it("should handle concurrent multi-agent access", async () => {
       const agents = ["agent-1", "agent-2", "agent-3"];

       // Create sessions for all agents
       const sessions = await Promise.all(
         agents.map((agentId, i) =>
           agentClient.createTerminalSession({
             taskId: `E2E-${i}`,
             agentId,
           })
         )
       );

       // Execute commands concurrently
       const results = await Promise.all(
         sessions.map((session) =>
           agentClient.executeCommand({
             sessionId: session.sessionId,
             command: "echo",
             args: ["hello"],
           })
         )
       );

       // All should succeed independently
       results.forEach((result) => {
         expect(result.success).toBe(true);
       });

       // Cleanup
       await Promise.all(
         sessions.map((session) => agentClient.closeSession(session.sessionId))
       );
     });
   });
   ```

**Coverage Goal**: Critical user journeys validated

---

## Performance Tests

**File**: `tests/performance/terminal-performance.test.ts`

### Benchmarks

```typescript
describe("Performance Tests", () => {
  it("should create session within 100ms", async () => {
    const start = Date.now();

    await manager.createSession("PERF-1", "agent");

    const duration = Date.now() - start;
    expect(duration).toBeLessThan(100);
  });

  it("should execute command with <500ms overhead", async () => {
    const session = await manager.createSession("PERF-2", "agent");

    const start = Date.now();
    await manager.executeCommand({
      sessionId: session.id,
      command: "echo",
      args: ["test"],
    });
    const duration = Date.now() - start;

    // Command itself is ~instant, overhead should be <500ms
    expect(duration).toBeLessThan(500);
  });

  it("should handle 50 concurrent sessions", async () => {
    const sessions = await Promise.all(
      Array.from({ length: 50 }, (_, i) =>
        manager.createSession(`PERF-${i}`, "agent")
      )
    );

    expect(sessions).toHaveLength(50);

    // Cleanup
    await Promise.all(sessions.map((s) => manager.closeSession(s.id)));
  });
});
```

---

## Test Execution

### Commands

```bash
# Run all tests
npm test

# Run specific test suites
npm test -- tests/unit/security/CommandValidator.test.ts
npm test -- tests/integration/terminal-execution.test.ts
npm test -- tests/e2e/mcp-terminal-workflow.test.ts

# Run with coverage
npm run test:coverage

# Run security tests only
npm test -- --testPathPattern=security

# Run mutation tests
npm run test:mutation
```

### Success Criteria

- ✅ All unit tests pass
- ✅ Line coverage ≥90%
- ✅ Branch coverage ≥90%
- ✅ Mutation score ≥75%
- ✅ All security tests pass (zero vulnerabilities)
- ✅ E2E workflow completes successfully
- ✅ Performance benchmarks met

---

## CI/CD Integration

### GitHub Actions Workflow

```yaml
- name: Run Terminal Access Tests
  run: |
    npm run test:coverage -- tests/unit/security/CommandValidator.test.ts
    npm run test:coverage -- tests/unit/mcp-server/TerminalSessionManager.test.ts
    npm run test:integration
    npm run test:security

- name: Check Coverage Thresholds
  run: |
    npx nyc check-coverage --lines 90 --branches 90 --functions 90

- name: Run Mutation Tests
  run: npm run test:mutation
```

---

## Test Data & Fixtures

### Test Fixtures

```typescript
// tests/fixtures/terminal-fixtures.ts

export const mockAllowedCommands = ["npm", "git", "node", "echo"];

export const mockSession: TerminalSession = {
  id: "test-session-123",
  taskId: "TASK-001",
  agentId: "test-agent",
  workingDirectory: "/workspace",
  environment: {},
  state: "idle",
  createdAt: new Date("2025-01-01"),
};

export const safeCommands = [
  { command: "npm", args: ["test"] },
  { command: "git", args: ["status"] },
  { command: "echo", args: ["hello"] },
];

export const dangerousCommands = [
  { command: "rm", args: ["-rf", "/"] },
  { command: "sudo", args: ["rm", "file"] },
  { command: "echo", args: ["test;rm -rf /"] },
];
```

---

## Test Coverage Report Format

```
File                          | % Stmts | % Branch | % Funcs | % Lines |
------------------------------|---------|----------|---------|---------|
CommandValidator.ts           |   95.00 |    93.75 |   100.0 |   95.00 |
TerminalSessionManager.ts     |   92.50 |    90.00 |   94.00 |   92.50 |
terminal-handlers.ts          |   88.00 |    85.00 |   90.00 |   88.00 |
------------------------------|---------|----------|---------|---------|
All files                     |   91.00 |    89.50 |   94.00 |   91.00 |
```

**Target**: All metrics ≥90%

---

**Test Plan Status**: ✅ Complete  
**Ready for**: Implementation with TDD approach  
**Next Step**: Begin Phase 2 - Implement types and CommandValidator with tests first
