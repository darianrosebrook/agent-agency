# MCP Tools Directory

This directory contains tool manifests that can be automatically discovered and used by the Agent Agency V3 MCP (Model Context Protocol) system.

## How Tool Discovery Works

The MCP server automatically discovers tools by:

1. **Scanning configured paths** for manifest files (`.json` or `.toml` files)
2. **Parsing manifest files** to extract tool definitions
3. **Validating tools** against CAWS compliance rules
4. **Registering tools** in the MCP registry for execution

## Tool Manifest Format

Each tool is defined by a JSON manifest file with the following structure:

```json
{
  "name": "tool-name",
  "version": "1.0.0",
  "description": "What the tool does",
  "author": "tool-author",
  "tool_type": "ToolType",
  "entry_point": "/path/to/executable",
  "capabilities": ["Capability1", "Capability2"],
  "parameters": {
    "required": [...],
    "optional": [...],
    "constraints": [...]
  },
  "output_schema": {...},
  "caws_compliance": {...},
  "metadata": {...}
}
```

## Tool Types

- `CodeGeneration` - Tools that generate code
- `CodeAnalysis` - Linting, formatting, static analysis
- `Testing` - Unit tests, integration tests, performance tests
- `Documentation` - Generate documentation, READMEs
- `Build` - Compilation, bundling, packaging
- `Deployment` - CI/CD, containerization, cloud deployment
- `Monitoring` - Logging, metrics, health checks
- `Utility` - General-purpose utilities

## Capabilities

Tools declare what operations they can perform:

- `FileRead` - Can read files from the filesystem
- `FileWrite` - Can write files to the filesystem
- `FileSystemAccess` - General filesystem access
- `CommandExecution` - Can execute shell commands
- `NetworkAccess` - Can make network requests
- `DatabaseAccess` - Can access databases
- `ImageProcessing` - Can process images
- `TextProcessing` - Can process text
- `CodeGeneration` - Can generate code
- `CodeAnalysis` - Can analyze code
- `TestExecution` - Can run tests
- `DocumentationGeneration` - Can generate documentation

## CAWS Compliance

Tools must declare their CAWS compliance status:

```json
{
  "caws_compliance": {
    "required_rules": ["code-quality", "security"],
    "strict_mode": true,
    "custom_validations": [...]
  }
}
```

## Example Tool Execution

Once discovered, tools can be executed through the MCP server:

```javascript
// Execute ESLint tool
const result = await mcpServer.executeTool({
  tool_id: "eslint-runner",
  parameters: {
    files: ["src/main.js", "src/utils.js"],
    format: "json"
  }
});

console.log("Linting passed:", result.output.passed);
```

## Adding New Tools

To add a new tool:

1. Create a JSON manifest file in this directory
2. Ensure the tool executable is available at the specified `entry_point`
3. Test the tool manifest with the MCP server
4. Add any required dependencies to the tool's environment

## Security Considerations

- Tools are executed in a sandboxed environment
- File system access is restricted to allowed paths
- Network access may be limited based on CAWS compliance
- Command execution is validated for dangerous patterns

## Current Tools

- **eslint.json** - JavaScript/TypeScript linting
- **prettier.json** - Code formatting for JS/TS/CSS/HTML
- **jest.json** - JavaScript/TypeScript testing

## Discovery Configuration

Tool discovery is configured in the MCP server config:

```json
{
  "tool_discovery": {
    "enable_auto_discovery": true,
    "discovery_paths": ["./tools", "./extensions"],
    "manifest_patterns": ["**/tool.json", "**/manifest.toml"],
    "discovery_interval_seconds": 60,
    "enable_validation": true,
    "enable_health_checks": true
  }
}
```
