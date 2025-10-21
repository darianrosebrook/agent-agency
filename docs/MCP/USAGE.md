# MCP Tool Ecosystem Usage Guide

## Overview

**✅ FULLY IMPLEMENTED** - The Model Context Protocol (MCP) integration provides a comprehensive tool ecosystem enabling external AI models and autonomous agents to leverage Agent Agency's sophisticated internal capabilities. This guide covers the 13 specialized tools across 7 categories for governance, verification, reasoning, and workflow management.

## Quick Start

### Prerequisites

- Node.js 18+ with ES modules support
- Local AI model (Gemma 3N recommended)
- Ollama for model hosting (optional)

### Installation

```bash
# Install dependencies
npm install

# Build the project
npm run build

# Start the MCP server
npm run mcp:start
```

### Basic Usage with Local AI

1. **Start the MCP Server:**

   ```bash
   npm run mcp:start
   ```

2. **Connect with Local AI Model:**
   Configure your local AI model (e.g., Gemma via Ollama) to connect to the MCP server using stdio transport.

3. **Available Resources:**

   - `agents://list` - List all registered agents
   - `tasks://queue` - View current task queue
   - `system://metrics` - System performance metrics
   - `memory://experiences/{agentId}` - Agent experience history

4. **Available Tools:**
   - `register_agent` - Register new agents
   - `submit_task` - Submit tasks for execution
   - `evaluate_code` - Evaluate code quality
   - `run_evaluation_loop` - Execute autonomous evaluation cycles

## MCP Resources

### Agent Resources

#### `agents://list`

Lists all registered agents with their status and capabilities.

```javascript
// Response format
{
  'agents': [
    {
      'id': 'agent_001',
      'name': 'Data Processor',
      'type': 'worker',
      'status': 'active',
      'capabilities': ['process', 'analyze']
    }
  ],
  'total': 1
}
```

#### `agent://{agentId}`

Retrieves detailed information about a specific agent.

#### `agents://capabilities/{agentId}`

Shows agent capabilities and proficiency levels.

#### `agents://relationships/{agentId}`

Displays collaboration history and relationships with other agents.

### Task Resources

#### `tasks://queue`

Shows current task queue with pending and running tasks.

#### `tasks://history/{agentId}`

Provides task execution history for a specific agent.

#### `task://{taskId}`

Detailed information about a specific task including status, payload, and results.

#### `tasks://metrics`

Task performance and success metrics across the system.

### System Resources

#### `system://metrics`

Real-time system health and performance metrics.

```javascript
{
  'totalAgents': 5,
  'activeAgents': 3,
  'totalTasks': 42,
  'completedTasks': 38,
  'failedTasks': 2,
  'averageTaskDuration': 1250,
  'systemUptime': 3600
}
```

#### `system://config`

Current system configuration settings.

#### `system://health`

Comprehensive system health assessment with component status.

#### `system://logs`

Recent system activity and error logs.

## MCP Tools

### Agent Management Tools

#### `register_agent`

Register a new agent with the system.

**Parameters:**

```javascript
{
  'name': 'Data Processor',
  'type': 'worker',
  'capabilities': ['process', 'analyze'],
  'metadata': {
    'version': '1.0.0',
    'description': 'Processes and analyzes data'
  }
}
```

**Response:**

```javascript
{
  'agentId': 'agent_001',
  'agent': {
    'id': 'agent_001',
    'name': 'Data Processor',
    // ... full agent object
  }
}
```

#### `update_agent`

Update an existing agent's information.

#### `get_agent`

Retrieve detailed agent information.

#### `list_agents`

List agents with optional filtering.

### Task Management Tools

#### `submit_task`

Submit a new task for execution.

**Parameters:**

```javascript
{
  'agentId': 'agent_001',
  'type': 'process',
  'payload': {
    'data': 'sample input',
    'options': { 'priority': 'high' }
  }
}
```

**Response:**

```javascript
{
  'taskId': 'task_001',
  'task': {
    'id': 'task_001',
    'agentId': 'agent_001',
    'status': 'pending',
    // ... full task object
  }
}
```

#### `get_task`

Retrieve task details.

#### `cancel_task`

Cancel a pending or running task.

#### `list_tasks`

List tasks with filtering options.

#### `retry_task`

Retry a failed task.

### Evaluation Tools

#### `evaluate_code`

Evaluate code quality with automated testing and linting.

**Parameters:**

```javascript
{
  'taskId': 'task_001',
  'projectDir': './project',
  'scripts': {
    'test': 'npm run test',
    'lint': 'npm run lint',
    'typecheck': 'npm run typecheck'
  },
  'iteration': 1
}
```

**Response:**

```javascript
{
  'taskId': 'task_001',
  'type': 'code',
  'iteration': 1,
  'status': 'pass',
  'score': 0.92,
  'criteria': [
    {
      'id': 'tests-pass',
      'description': 'Unit tests pass',
      'weight': 0.4,
      'passed': true,
      'score': 1.0
    }
    // ... more criteria
  ],
  'nextActions': [],
  'timestamp': '2024-01-01T12:00:00.000Z'
}
```

#### `evaluate_text`

Evaluate text quality and adherence to requirements.

#### `evaluate_design`

Evaluate design token compliance and consistency.

#### `run_evaluation_loop`

Execute a complete autonomous evaluation loop.

### System Tools

#### `get_system_metrics`

Retrieve current system performance metrics.

#### `perform_health_check`

Execute comprehensive system health assessment.

#### `clear_system_cache`

Clear system caches and temporary data.

#### `backup_system_data`

Create backup of system data and configuration.

#### `get_system_config`

Retrieve current system configuration.

#### `update_system_config`

Update system configuration parameters.

## Autonomous Operation Examples

### Basic Agent Registration and Task Submission

```javascript
// 1. Register an agent
const registerResult = await mcp.callTool('register_agent', {
  name: 'Code Reviewer',
  type: 'worker',
  capabilities: ['review', 'analyze'],
  metadata: { version: '1.0.0' },
});

// 2. Submit a task
const taskResult = await mcp.callTool('submit_task', {
  agentId: registerResult.agentId,
  type: 'review',
  payload: {
    code: 'function example() { return true; }',
    criteria: ['readability', 'best-practices'],
  },
});

// 3. Monitor task progress
const taskStatus = await mcp.readResource(`task://${taskResult.taskId}`);
```

### Autonomous Code Improvement Loop

```javascript
async function improveCode(code, requirements) {
  let iteration = 1;
  let bestScore = 0;
  let bestCode = code;

  while (iteration <= 3) {
    // Submit improvement task
    const taskResult = await mcp.callTool('submit_task', {
      agentId: 'code-improver-agent',
      type: 'improve',
      payload: { code: bestCode, requirements, iteration },
    });

    // Wait for completion (simplified)
    let task = await mcp.readResource(`task://${taskResult.taskId}`);
    while (task.status !== 'completed') {
      await new Promise((resolve) => setTimeout(resolve, 1000));
      task = await mcp.readResource(`task://${taskResult.taskId}`);
    }

    // Evaluate the result
    const evaluation = await mcp.callTool('evaluate_code', {
      taskId: task.id,
      iteration,
    });

    if (evaluation.score > bestScore) {
      bestScore = evaluation.score;
      bestCode = task.result.improvedCode;
    }

    // Check satisficing condition
    if (evaluation.score >= 0.85) {
      break;
    }

    iteration++;
  }

  return { finalCode: bestCode, finalScore: bestScore };
}
```

### Self-Monitoring Agent System

```javascript
class SelfMonitoringAgent {
  constructor(agentId) {
    this.agentId = agentId;
    this.performanceHistory = [];
  }

  async executeTask(taskType, payload) {
    // Submit task
    const taskResult = await mcp.callTool('submit_task', {
      agentId: this.agentId,
      type: taskType,
      payload,
    });

    // Monitor execution
    const startTime = Date.now();
    let task = await mcp.readResource(`task://${taskResult.taskId}`);

    while (task.status === 'pending' || task.status === 'running') {
      await new Promise((resolve) => setTimeout(resolve, 500));
      task = await mcp.readResource(`task://${taskResult.taskId}`);
    }

    const executionTime = Date.now() - startTime;

    // Record performance
    this.performanceHistory.push({
      taskId: task.id,
      taskType,
      success: task.status === 'completed',
      executionTime,
      timestamp: new Date().toISOString(),
    });

    // Analyze performance trends
    if (this.performanceHistory.length >= 10) {
      await this.analyzePerformanceTrends();
    }

    return task;
  }

  async analyzePerformanceTrends() {
    const recentTasks = this.performanceHistory.slice(-10);
    const successRate =
      recentTasks.filter((t) => t.success).length / recentTasks.length;
    const avgExecutionTime =
      recentTasks.reduce((sum, t) => sum + t.executionTime, 0) /
      recentTasks.length;

    // Trigger self-improvement if performance is declining
    if (successRate < 0.8) {
      await this.triggerSelfImprovement();
    }
  }

  async triggerSelfImprovement() {
    // Submit self-improvement task
    await mcp.callTool('submit_task', {
      agentId: this.agentId,
      type: 'self-improve',
      payload: {
        analysis: 'Performance declining',
        performanceHistory: this.performanceHistory,
      },
    });
  }
}
```

## Configuration

### MCP Server Configuration

```typescript
interface MCPServerConfig {
  orchestrator: AgentOrchestrator;
  evaluationConfig?: {
    minScore: number; // Minimum acceptable score (0.85)
    mandatoryGates: string[]; // Required quality gates
    iterationPolicy: {
      maxIterations: number; // Maximum refinement cycles (3)
      minDeltaToContinue: number; // Minimum improvement needed (0.02)
      noChangeBudget: number; // Plateau tolerance (1)
    };
  };
}
```

### Environment Variables

```bash
# MCP Server Configuration
MCP_PORT=3001
MCP_HOST=localhost

# Evaluation Configuration
MCP_MIN_SCORE=0.85
MCP_MAX_ITERATIONS=3

# System Configuration
MCP_CACHE_TTL=300
MCP_MAX_CONCURRENT_TASKS=10
```

## Troubleshooting

### Common Issues

#### MCP Server Won't Start

```bash
# Check Node.js version
node --version  # Should be 18+

# Verify dependencies
npm ls @modelcontextprotocol/sdk

# Check for port conflicts
lsof -i :3001
```

#### Tool Execution Fails

```bash
# Verify agent capabilities
const agent = await mcp.readResource(`agent://${agentId}`);
console.log('Agent capabilities:', agent.capabilities);

# Check task payload format
const taskSchema = await mcp.listTools();
const tool = taskSchema.tools.find(t => t.name === 'submit_task');
console.log('Task schema:', tool.inputSchema);
```

#### Evaluation Not Working

```bash
# Check evaluation configuration
const config = await mcp.readResource('system://config');
console.log('Evaluation config:', config.evaluation);

# Verify file permissions
ls -la /path/to/artifact
```

### Debug Mode

Enable debug logging for troubleshooting:

```bash
# Set environment variable
export DEBUG=mcp:*

# Or configure in code
const logger = new Logger('MCP', 'debug');
```

### Performance Monitoring

Monitor MCP server performance:

```bash
# Get system metrics
const metrics = await mcp.readResource('system://metrics');

# Monitor evaluation performance
const health = await mcp.readResource('system://health');

# Check tool execution times
const logs = await mcp.readResource('system://logs');
```

## Advanced Usage

### Custom Evaluation Criteria

Extend the evaluation system with custom criteria:

```typescript
class CustomEvaluator extends BaseEvaluator {
  async evaluate(params: EvaluationParams): Promise<EvaluationReport> {
    // Implement custom evaluation logic
    const customCriteria = [
      {
        id: 'custom-metric',
        description: 'Custom quality metric',
        weight: 0.3,
        passed: await this.checkCustomMetric(params.artifactPath),
        score: await this.scoreCustomMetric(params.artifactPath),
      },
    ];

    return {
      taskId: params.taskId,
      artifactPaths: [params.artifactPath],
      status: 'completed',
      score: customCriteria.reduce((s, c) => s + c.score * c.weight, 0),
      criteria: customCriteria,
      iterations: params.iterations,
      timestamp: new Date().toISOString(),
    };
  }
}
```

### Multi-Agent Coordination

Coordinate multiple agents for complex workflows:

```typescript
class MultiAgentCoordinator {
  async executeWorkflow(workflow) {
    const results = [];

    for (const step of workflow.steps) {
      // Find suitable agent
      const agents = await mcp.callTool('list_agents', {
        capability: step.requiredCapability,
      });

      const agent = this.selectBestAgent(agents.agents, step);

      // Submit task to selected agent
      const taskResult = await mcp.callTool('submit_task', {
        agentId: agent.id,
        type: step.type,
        payload: { ...step.payload, previousResults: results },
      });

      results.push(await this.waitForTaskCompletion(taskResult.taskId));
    }

    return results;
  }

  selectBestAgent(agents, step) {
    // Implement agent selection logic based on
    // performance history, current load, capabilities, etc.
    return agents[0]; // Simplified
  }
}
```

## Security Considerations

### Access Control

- MCP tools validate agent permissions before execution
- Resource access is filtered based on agent capabilities
- Sensitive operations require explicit authorization

### Input Validation

- All tool parameters are validated against schemas
- File paths are sanitized to prevent directory traversal
- Payload size limits prevent resource exhaustion

### Audit Logging

- All tool executions are logged with timestamps
- Resource access is tracked for compliance
- Failed operations include error details for debugging

## Contributing

### Adding New Tools

1. Implement tool logic in appropriate category class
2. Add tool schema with proper validation
3. Update documentation with usage examples
4. Add unit tests for tool functionality

### Adding New Resources

1. Implement resource handler in ResourceManager
2. Add resource URI pattern and schema
3. Update access control and filtering
4. Document resource format and usage

### Extending Evaluation

1. Create new evaluator extending BaseEvaluator
2. Implement evaluation logic and criteria
3. Add configuration options
4. Update satisficing logic if needed

## Support

For issues and questions:

- Check the troubleshooting section above
- Review the integration test examples
- Examine system logs for error details
- Refer to the technical architecture documentation

## License

This MCP integration is part of the Agent Agency project and follows the same MIT license.
