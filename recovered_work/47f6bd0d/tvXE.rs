//! Production Documentation
//!
//! Comprehensive documentation generation for deployment guides,
//! API documentation, architecture docs, and operational procedures.

use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Documentation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationConfig {
    pub output_directory: PathBuf,
    pub include_api_docs: bool,
    pub include_deployment_guide: bool,
    pub include_architecture_docs: bool,
    pub include_operations_guide: bool,
    pub generate_markdown: bool,
    pub generate_html: bool,
    pub include_examples: bool,
    pub author: String,
    pub version: String,
}

/// API documentation generator
pub struct ApiDocs {
    config: DocumentationConfig,
}

impl ApiDocs {
    pub fn new(config: DocumentationConfig) -> Self {
        Self { config }
    }

    /// Generate API documentation
    pub async fn generate_api_docs(&self) -> Result<String, DocumentationError> {
        let mut docs = String::new();

        docs.push_str("# Agent Agency V3 API Documentation\n\n");
        docs.push_str(&format!("**Version:** {}\n", self.config.version));
        docs.push_str(&format!("**Generated:** {}\n\n", Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));

        // REST API Endpoints
        docs.push_str("## REST API Endpoints\n\n");

        let endpoints = vec![
            ("POST /api/v1/tasks", "Submit a task for autonomous execution"),
            ("GET /api/v1/tasks/{id}", "Get task status and results"),
            ("GET /api/v1/tasks", "List all tasks"),
            ("DELETE /api/v1/tasks/{id}", "Cancel a running task"),
            ("GET /api/v1/health", "Health check endpoint"),
            ("GET /api/v1/metrics", "System metrics"),
        ];

        for (endpoint, description) in endpoints {
            docs.push_str(&format!("- **`{}`** - {}\n", endpoint, description));
        }

        docs.push_str("\n### Task Submission\n\n");
        docs.push_str("```http\n");
        docs.push_str("POST /api/v1/tasks\n");
        docs.push_str("Content-Type: application/json\n");
        docs.push_str("Authorization: Bearer <jwt_token>\n\n");
        docs.push_str("{\n");
        docs.push_str("  \"description\": \"Build a user authentication system with JWT tokens\",\n");
        docs.push_str("  \"risk_tier\": \"high\",\n");
        docs.push_str("  \"context\": \"Additional requirements and constraints\"\n");
        docs.push_str("}\n");
        docs.push_str("```\n\n");

        docs.push_str("**Response:**\n");
        docs.push_str("```json\n");
        docs.push_str("{\n");
        docs.push_str("  \"task_id\": \"550e8400-e29b-41d4-a716-446655440000\",\n");
        docs.push_str("  \"status\": \"accepted\",\n");
        docs.push_str("  \"message\": \"Task submitted for autonomous execution\",\n");
        docs.push_str("  \"estimated_completion\": \"2024-01-15T15:30:00Z\"\n");
        docs.push_str("}\n");
        docs.push_str("```\n\n");

        // WebSocket API
        docs.push_str("## WebSocket API\n\n");
        docs.push_str("Connect to `ws://localhost:8080/ws/tasks` for real-time task monitoring.\n\n");

        docs.push_str("### Message Types\n\n");

        let ws_messages = vec![
            ("subscribe", "Subscribe to task events", "{\"type\": \"subscribe\", \"task_id\": \"...\"}"),
            ("status", "Get task status", "{\"type\": \"get_status\", \"task_id\": \"...\"}"),
            ("events", "Receive execution events", "Automatic after subscription"),
        ];

        for (msg_type, description, example) in ws_messages {
            docs.push_str(&format!("- **{}** - {}\n", msg_type, description));
            docs.push_str(&format!("  ```json\n  {}\n  ```\n\n", example));
        }

        // CLI Interface
        docs.push_str("## CLI Interface\n\n");
        docs.push_str("```bash\n");
        docs.push_str("# Submit a task\n");
        docs.push_str("agent-agency submit \"Build a REST API\" --watch\n\n");
        docs.push_str("# Check task status\n");
        docs.push_str("agent-agency status 550e8400-e29b-41d4-a716-446655440000\n\n");
        docs.push_str("# Get results\n");
        docs.push_str("agent-agency result 550e8400-e29b-41d4-a716-446655440000 --save-artifacts ./output\n");
        docs.push_str("```\n\n");

        // MCP Server
        docs.push_str("## MCP (Model Context Protocol) Server\n\n");
        docs.push_str("Provides integration with IDEs and AI assistants.\n\n");
        docs.push_str("**Tools Available:**\n");
        docs.push_str("- `submit_task` - Submit tasks for execution\n");
        docs.push_str("- `get_task_status` - Monitor task progress\n");
        docs.push_str("- `list_tasks` - View all tasks\n");
        docs.push_str("- `cancel_task` - Stop running tasks\n\n");

        // Authentication
        docs.push_str("## Authentication\n\n");
        docs.push_str("### JWT Token Authentication\n");
        docs.push_str("```http\n");
        docs.push_str("POST /api/v1/auth/login\n");
        docs.push_str("Content-Type: application/json\n\n");
        docs.push_str("{\n");
        docs.push_str("  \"username\": \"developer\",\n");
        docs.push_str("  \"password\": \"secure_password\"\n");
        docs.push_str("}\n");
        docs.push_str("```\n\n");

        docs.push_str("### API Key Authentication\n");
        docs.push_str("```http\n");
        docs.push_str("GET /api/v1/tasks\n");
        docs.push_str("X-API-Key: your-api-key-here\n");
        docs.push_str("```\n\n");

        // Error Codes
        docs.push_str("## Error Codes\n\n");
        let error_codes = vec![
            ("400", "Bad Request", "Invalid input parameters"),
            ("401", "Unauthorized", "Authentication required"),
            ("403", "Forbidden", "Insufficient permissions"),
            ("404", "Not Found", "Resource does not exist"),
            ("429", "Too Many Requests", "Rate limit exceeded"),
            ("500", "Internal Server Error", "Server-side error"),
        ];

        for (code, title, description) in error_codes {
            docs.push_str(&format!("- **`{}` {}** - {}\n", code, title, description));
        }

        docs.push_str("\n");

        Ok(docs)
    }
}

/// Deployment guide generator
pub struct DeploymentGuide {
    config: DocumentationConfig,
}

impl DeploymentGuide {
    pub fn new(config: DocumentationConfig) -> Self {
        Self { config }
    }

    /// Generate deployment guide
    pub async fn generate_deployment_guide(&self) -> Result<String, DocumentationError> {
        let mut guide = String::new();

        guide.push_str("# Agent Agency V3 Deployment Guide\n\n");
        guide.push_str(&format!("**Version:** {}\n", self.config.version));
        guide.push_str(&format!("**Author:** {}\n", self.config.author));
        guide.push_str(&format!("**Generated:** {}\n\n", Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));

        // Prerequisites
        guide.push_str("## Prerequisites\n\n");
        guide.push_str("### System Requirements\n");
        guide.push_str("- **OS:** Linux, macOS, or Windows\n");
        guide.push_str("- **CPU:** 4+ cores recommended\n");
        guide.push_str("- **RAM:** 8GB minimum, 16GB recommended\n");
        guide.push_str("- **Storage:** 50GB available space\n");
        guide.push_str("- **Network:** Stable internet connection\n\n");

        guide.push_str("### Software Dependencies\n");
        guide.push_str("- **Rust:** 1.70+ (https://rustup.rs/)\n");
        guide.push_str("- **PostgreSQL:** 13+ for data persistence\n");
        guide.push_str("- **Redis:** 6+ for caching (optional)\n");
        guide.push_str("- **Docker:** For containerized deployment\n\n");

        // Installation
        guide.push_str("## Installation\n\n");
        guide.push_str("### From Source\n");
        guide.push_str("```bash\n");
        guide.push_str("# Clone the repository\n");
        guide.push_str("git clone https://github.com/your-org/agent-agency.git\n");
        guide.push_str("cd agent-agency/iterations/v3\n\n");
        guide.push_str("# Build the project\n");
        guide.push_str("cargo build --release\n\n");
        guide.push_str("# Run tests\n");
        guide.push_str("cargo test\n");
        guide.push_str("```\n\n");

        guide.push_str("### Using Docker\n");
        guide.push_str("```bash\n");
        guide.push_str("# Build the Docker image\n");
        guide.push_str("docker build -t agent-agency-v3 .\n\n");
        guide.push_str("# Run the container\n");
        guide.push_str("docker run -p 3000:3000 -p 8080:8080 agent-agency-v3\n");
        guide.push_str("```\n\n");

        // Configuration
        guide.push_str("## Configuration\n\n");
        guide.push_str("### Environment Variables\n");
        guide.push_str("```bash\n");
        guide.push_str("# Database\n");
        guide.push_str("export DATABASE_URL=\"postgres://user:password@localhost/agent_agency\"\n\n");
        guide.push_str("# Security\n");
        guide.push_str("export JWT_SECRET=\"your-super-secure-jwt-secret-key\"\n");
        guide.push_str("export API_KEYS=\"key1,key2,key3\"\n\n");
        guide.push_str("# Server\n");
        guide.push_str("export SERVER_HOST=\"0.0.0.0\"\n");
        guide.push_str("export SERVER_PORT=\"3000\"\n");
        guide.push_str("export WS_PORT=\"8080\"\n");
        guide.push_str("```\n\n");

        guide.push_str("### Configuration File\n");
        guide.push_str("Create `config.toml`:\n");
        guide.push_str("```toml\n");
        guide.push_str("[server]\n");
        guide.push_str("host = \"0.0.0.0\"\n");
        guide.push_str("port = 3000\n");
        guide.push_str("workers = 4\n\n");
        guide.push_str("[database]\n");
        guide.push_str("url = \"postgres://user:password@localhost/agent_agency\"\n");
        guide.push_str("max_connections = 10\n\n");
        guide.push_str("[security]\n");
        guide.push_str("jwt_secret = \"your-secret-key\"\n");
        guide.push_str("enable_auth = true\n");
        guide.push_str("```\n\n");

        // Database Setup
        guide.push_str("## Database Setup\n\n");
        guide.push_str("```sql\n");
        guide.push_str("-- Create database\n");
        guide.push_str("CREATE DATABASE agent_agency;\n\n");
        guide.push_str("-- Run migrations\n");
        guide.push_str("cargo run --bin migrate\n");
        guide.push_str("```\n\n");

        // Running the Service
        guide.push_str("## Running the Service\n\n");
        guide.push_str("### Development Mode\n");
        guide.push_str("```bash\n");
        guide.push_str("cargo run --bin agent-agency\n");
        guide.push_str("```\n\n");

        guide.push_str("### Production Mode\n");
        guide.push_str("```bash\n");
        guide.push_str("# Build optimized binary\n");
        guide.push_str("cargo build --release\n\n");
        guide.push_str("# Run the service\n");
        guide.push_str("./target/release/agent-agency\n");
        guide.push_str("```\n\n");

        guide.push_str("### Using Systemd\n");
        guide.push_str("Create `/etc/systemd/system/agent-agency.service`:\n");
        guide.push_str("```ini\n");
        guide.push_str("[Unit]\n");
        guide.push_str("Description=Agent Agency V3\n");
        guide.push_str("After=network.target postgresql.service\n\n");
        guide.push_str("[Service]\n");
        guide.push_str("Type=simple\n");
        guide.push_str("User=agent-agency\n");
        guide.push_str("WorkingDirectory=/opt/agent-agency\n");
        guide.push_str("ExecStart=/opt/agent-agency/target/release/agent-agency\n");
        guide.push_str("Restart=on-failure\n\n");
        guide.push_str("[Install]\n");
        guide.push_str("WantedBy=multi-user.target\n");
        guide.push_str("```\n\n");

        // Monitoring
        guide.push_str("## Monitoring\n\n");
        guide.push_str("### Health Checks\n");
        guide.push_str("- **HTTP:** `GET /health` - Service health status\n");
        guide.push_str("- **Metrics:** `GET /metrics` - Prometheus-compatible metrics\n");
        guide.push_str("- **Logs:** Structured JSON logging to stdout\n\n");

        guide.push_str("### Key Metrics to Monitor\n");
        guide.push_str("- **Task Success Rate:** Percentage of successful task executions\n");
        guide.push_str("- **Average Execution Time:** Time to complete tasks\n");
        guide.push_str("- **Error Rate:** Rate of errors across components\n");
        guide.push_str("- **Resource Usage:** CPU, memory, and disk utilization\n");
        guide.push_str("- **Quality Scores:** Average quality scores for completed tasks\n\n");

        // Troubleshooting
        guide.push_str("## Troubleshooting\n\n");
        guide.push_str("### Common Issues\n\n");
        guide.push_str("**Database Connection Failed**\n");
        guide.push_str("```bash\n");
        guide.push_str("# Check database connectivity\n");
        guide.push_str("psql $DATABASE_URL -c \"SELECT 1;\"\n\n");
        guide.push_str("# Verify database exists\n");
        guide.push_str("psql -l | grep agent_agency\n");
        guide.push_str("```\n\n");

        guide.push_str("**Port Already in Use**\n");
        guide.push_str("```bash\n");
        guide.push_str("# Find process using port\n");
        guide.push_str("lsof -i :3000\n\n");
        guide.push_str("# Kill process\n");
        guide.push_str("kill -9 <PID>\n");
        guide.push_str("```\n\n");

        guide.push_str("**Authentication Issues**\n");
        guide.push_str("- Verify JWT_SECRET environment variable\n");
        guide.push_str("- Check API key configuration\n");
        guide.push_str("- Validate user credentials\n\n");

        // Security Considerations
        guide.push_str("## Security Considerations\n\n");
        guide.push_str("### Network Security\n");
        guide.push_str("- Use HTTPS in production\n");
        guide.push_str("- Configure firewall rules\n");
        guide.push_str("- Use internal networking for database\n\n");

        guide.push_str("### Authentication & Authorization\n");
        guide.push_str("- Enable JWT authentication\n");
        guide.push_str("- Configure role-based access control\n");
        guide.push_str("- Rotate secrets regularly\n\n");

        guide.push_str("### Data Protection\n");
        guide.push_str("- Encrypt sensitive data at rest\n");
        guide.push_str("- Use secure database connections\n");
        guide.push_str("- Implement audit logging\n\n");

        Ok(guide)
    }
}

/// Architecture documentation generator
pub struct ArchitectureDocs {
    config: DocumentationConfig,
}

impl ArchitectureDocs {
    pub fn new(config: DocumentationConfig) -> Self {
        Self { config }
    }

    /// Generate architecture documentation
    pub async fn generate_architecture_docs(&self) -> Result<String, DocumentationError> {
        let mut docs = String::new();

        docs.push_str("# Agent Agency V3 Architecture\n\n");
        docs.push_str(&format!("**Version:** {}\n", self.config.version));
        docs.push_str(&format!("**Generated:** {}\n\n", Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));

        // Overview
        docs.push_str("## Overview\n\n");
        docs.push_str("Agent Agency V3 is a production-ready autonomous AI development platform that can understand natural language task descriptions and execute them with constitutional AI governance.\n\n");

        docs.push_str("## Core Principles\n\n");
        docs.push_str("- **Constitutional AI Governance:** Every decision reviewed by AI judges\n");
        docs.push_str("- **Quality-First Development:** Standards enforcement prevents technical debt\n");
        docs.push_str("- **Complete Traceability:** Full audit trail of all decisions and artifacts\n");
        docs.push_str("- **Multi-Interface Architecture:** Tool-agnostic integration\n\n");

        // System Architecture
        docs.push_str("## System Architecture\n\n");
        docs.push_str("```mermaid\n");
        docs.push_str("graph TB\n");
        docs.push_str("    A[User Interfaces] --> B[Orchestration Layer]\n");
        docs.push_str("    B --> C[Planning Agent]\n");
        docs.push_str("    B --> D[Council System]\n");
        docs.push_str("    B --> E[Worker Pool]\n");
        docs.push_str("    B --> F[Quality Gates]\n");
        docs.push_str("    B --> G[Artifact Management]\n");
        docs.push_str("    B --> H[Progress Tracking]\n");
        docs.push_str("    E --> I[Autonomous Executor]\n");
        docs.push_str("    I --> J[Refinement Loop]\n");
        docs.push_str("    K[Data Layer] --> L[(PostgreSQL)]\n");
        docs.push_str("    K --> M[(Redis)]\n");
        docs.push_str("    K --> N[(File System)]\n");
        docs.push_str("```\n\n");

        // Component Descriptions
        docs.push_str("## Core Components\n\n");

        let components = vec![
            ("Planning Agent", "Converts natural language to structured working specifications"),
            ("Council System", "Multi-judge constitutional AI governance and decision making"),
            ("Worker Pool", "Scalable execution of tasks with intelligent routing"),
            ("Quality Gates", "CAWS compliance, linting, testing, coverage, and mutation analysis"),
            ("Autonomous Executor", "Coordinates execution with real-time progress tracking"),
            ("Artifact Management", "Versioned storage and retrieval of execution artifacts"),
            ("Progress Tracking", "Real-time monitoring and event streaming"),
        ];

        for (component, description) in components {
            docs.push_str(&format!("### {}\n{}\n\n", component, description));
        }

        // Data Flow
        docs.push_str("## Data Flow\n\n");
        docs.push_str("1. **Task Intake:** Natural language task received via API, CLI, or MCP\n");
        docs.push_str("2. **Planning:** LLM generates working specification with acceptance criteria\n");
        docs.push_str("3. **Constitutional Review:** Council judges validate plan for compliance\n");
        docs.push_str("4. **Worker Assignment:** Task routed to appropriate worker based on capabilities\n");
        docs.push_str("5. **Execution:** Code generation, testing, and validation with progress tracking\n");
        docs.push_str("6. **Quality Gates:** Comprehensive quality assurance and standards enforcement\n");
        docs.push_str("7. **Refinement:** Council-directed improvements based on quality feedback\n");
        docs.push_str("8. **Artifact Storage:** Versioned storage with complete provenance\n\n");

        // Interface Layer
        docs.push_str("## Interface Layer\n\n");
        docs.push_str("### REST API\n");
        docs.push_str("- **Purpose:** Web applications, API integrations, automated systems\n");
        docs.push_str("- **Endpoints:** Task submission, status monitoring, results retrieval\n");
        docs.push_str("- **Authentication:** JWT tokens and API keys\n\n");

        docs.push_str("### CLI Interface\n");
        docs.push_str("- **Purpose:** Terminal users, scripts, CI/CD pipelines\n");
        docs.push_str("- **Features:** Progress bars, interactive monitoring, formatted output\n");
        docs.push_str("- **Commands:** submit, status, list, result, cancel, metrics\n\n");

        docs.push_str("### MCP Server\n");
        docs.push_str("- **Purpose:** IDE integration, AI assistants, development tools\n");
        docs.push_str("- **Protocol:** Model Context Protocol for tool integration\n");
        docs.push_str("- **Tools:** Task execution, monitoring, and control\n\n");

        docs.push_str("### WebSocket API\n");
        docs.push_str("- **Purpose:** Real-time dashboards, live monitoring systems\n");
        docs.push_str("- **Features:** Event streaming, interactive control, low-latency updates\n");
        docs.push_str("- **Messages:** Task events, status updates, control commands\n\n");

        // Security Architecture
        docs.push_str("## Security Architecture\n\n");
        docs.push_str("### Authentication\n");
        docs.push_str("- JWT token-based authentication\n");
        docs.push_str("- API key authentication for service accounts\n");
        docs.push_str("- Session management with automatic expiration\n\n");

        docs.push_str("### Authorization\n");
        docs.push_str("- Role-based access control (RBAC)\n");
        docs.push_str("- Resource-specific permissions\n");
        docs.push_str("- Context-aware authorization decisions\n\n");

        docs.push_str("### Input Validation\n");
        docs.push_str("- Comprehensive input sanitization\n");
        docs.push_str("- XSS and injection attack prevention\n");
        docs.push_str("- Request size and rate limiting\n\n");

        // Data Architecture
        docs.push_str("## Data Architecture\n\n");
        docs.push_str("### Storage Layers\n");
        docs.push_str("- **PostgreSQL:** Structured data, user accounts, task metadata\n");
        docs.push_str("- **Redis:** Caching, session storage, real-time metrics\n");
        docs.push_str("- **File System:** Artifact storage, logs, configuration files\n\n");

        docs.push_str("### Data Flow Patterns\n");
        docs.push_str("- **CQRS:** Command Query Responsibility Segregation for task operations\n");
        docs.push_str("- **Event Sourcing:** Complete audit trail of all state changes\n");
        docs.push_str("- **Eventual Consistency:** Asynchronous updates with eventual consistency\n\n");

        // Scalability Design
        docs.push_str("## Scalability Design\n\n");
        docs.push_str("### Horizontal Scaling\n");
        docs.push_str("- Stateless application servers\n");
        docs.push_str("- Database connection pooling\n");
        docs.push_str("- Distributed worker pools\n\n");

        docs.push_str("### Performance Optimizations\n");
        docs.push_str("- Response caching and compression\n");
        docs.push_str("- Asynchronous processing\n");
        docs.push_str("- Connection multiplexing\n\n");

        docs.push_str("### Resource Management\n");
        docs.push_str("- Memory pooling and garbage collection\n");
        docs.push_str("- CPU utilization monitoring\n");
        docs.push_str("- Automatic scaling based on load\n\n");

        // Monitoring & Observability
        docs.push_str("## Monitoring & Observability\n\n");
        docs.push_str("### Metrics Collection\n");
        docs.push_str("- **Task Metrics:** Success rates, execution times, quality scores\n");
        docs.push_str("- **System Metrics:** CPU, memory, disk, network utilization\n");
        docs.push_str("- **Business Metrics:** User engagement, feature usage\n\n");

        docs.push_str("### Logging Strategy\n");
        docs.push_str("- Structured JSON logging\n");
        docs.push_str("- Log levels: DEBUG, INFO, WARN, ERROR, FATAL\n");
        docs.push_str("- Contextual information in all log entries\n\n");

        docs.push_str("### Alerting\n");
        docs.push_str("- Error rate thresholds\n");
        docs.push_str("- Performance degradation alerts\n");
        docs.push_str("- Security incident notifications\n\n");

        // Deployment Patterns
        docs.push_str("## Deployment Patterns\n\n");
        docs.push_str("### Docker Containerization\n");
        docs.push_str("```dockerfile\n");
        docs.push_str("FROM rust:1.70-slim\n");
        docs.push_str("COPY . /app\n");
        docs.push_str("RUN cargo build --release\n");
        docs.push_str("EXPOSE 3000 8080\n");
        docs.push_str("CMD [\"./target/release/agent-agency\"]\n");
        docs.push_str("```\n\n");

        docs.push_str("### Kubernetes Deployment\n");
        docs.push_str("- **Horizontal Pod Autoscaling** based on CPU/memory metrics\n");
        docs.push_str("- **Rolling Updates** for zero-downtime deployments\n");
        docs.push_str("- **ConfigMaps and Secrets** for configuration management\n\n");

        docs.push_str("### Cloud Deployment\n");
        docs.push_str("- **AWS ECS/Fargate:** Container orchestration\n");
        docs.push_str("- **Google Cloud Run:** Serverless deployment\n");
        docs.push_str("- **Azure Container Apps:** Managed containers\n\n");

        Ok(docs)
    }
}

/// Comprehensive documentation generator
pub struct DocumentationGenerator {
    config: DocumentationConfig,
    api_docs: ApiDocs,
    deployment_guide: DeploymentGuide,
    architecture_docs: ArchitectureDocs,
}

impl DocumentationGenerator {
    pub fn new(config: DocumentationConfig) -> Self {
        Self {
            api_docs: ApiDocs::new(config.clone()),
            deployment_guide: DeploymentGuide::new(config.clone()),
            architecture_docs: ArchitectureDocs::new(config.clone()),
            config,
        }
    }

    /// Generate all documentation
    pub async fn generate_all_docs(&self) -> Result<HashMap<String, String>, DocumentationError> {
        let mut docs = HashMap::new();

        if self.config.include_api_docs {
            docs.insert("api.md".to_string(), self.api_docs.generate_api_docs().await?);
        }

        if self.config.include_deployment_guide {
            docs.insert("deployment.md".to_string(), self.deployment_guide.generate_deployment_guide().await?);
        }

        if self.config.include_architecture_docs {
            docs.insert("architecture.md".to_string(), self.architecture_docs.generate_architecture_docs().await?);
        }

        // Generate index
        let index = self.generate_index(&docs).await?;
        docs.insert("README.md".to_string(), index);

        Ok(docs)
    }

    /// Generate documentation index
    async fn generate_index(&self, docs: &HashMap<String, String>) -> Result<String, DocumentationError> {
        let mut index = String::new();

        index.push_str("# Agent Agency V3 Documentation\n\n");
        index.push_str(&format!("**Version:** {}\n", self.config.version));
        index.push_str(&format!("**Author:** {}\n", self.config.author));
        index.push_str(&format!("**Generated:** {}\n\n", Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));

        index.push_str("## Overview\n\n");
        index.push_str("Agent Agency V3 is a production-ready autonomous AI development platform that can understand natural language task descriptions and execute them with constitutional AI governance.\n\n");

        index.push_str("## Documentation Index\n\n");

        if docs.contains_key("api.md") {
            index.push_str("- **[API Documentation](api.md)** - Complete API reference for all interfaces\n");
        }

        if docs.contains_key("deployment.md") {
            index.push_str("- **[Deployment Guide](deployment.md)** - Installation, configuration, and production deployment\n");
        }

        if docs.contains_key("architecture.md") {
            index.push_str("- **[Architecture Docs](architecture.md)** - System design, data flow, and scaling patterns\n");
        }

        index.push_str("\n## Quick Start\n\n");
        index.push_str("```bash\n");
        index.push_str("# Install dependencies\n");
        index.push_str("curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh\n");
        index.push_str("source ~/.cargo/env\n\n");
        index.push_str("# Clone and build\n");
        index.push_str("git clone <repository-url>\n");
        index.push_str("cd agent-agency/iterations/v3\n");
        index.push_str("cargo build --release\n\n");
        index.push_str("# Configure environment\n");
        index.push_str("export DATABASE_URL=\"postgres://user:pass@localhost/agent_agency\"\n");
        index.push_str("export JWT_SECRET=\"your-secret-key\"\n\n");
        index.push_str("# Run the service\n");
        index.push_str("./target/release/agent-agency\n");
        index.push_str("```\n\n");

        index.push_str("## Key Features\n\n");
        index.push_str("- **Autonomous Execution:** Natural language to production code\n");
        index.push_str("- **Constitutional AI:** Ethical governance and compliance\n");
        index.push_str("- **Quality Assurance:** Comprehensive testing and validation\n");
        index.push_str("- **Multi-Interface:** REST, CLI, MCP, WebSocket support\n");
        index.push_str("- **Production Ready:** Monitoring, security, and scalability\n\n");

        index.push_str("## Architecture Overview\n\n");
        index.push_str("```mermaid\n");
        index.push_str("graph LR\n");
        index.push_str("    A[User] --> B[Interfaces]\n");
        index.push_str("    B --> C[Orchestration]\n");
        index.push_str("    C --> D[Planning]\n");
        index.push_str("    C --> E[Council]\n");
        index.push_str("    C --> F[Execution]\n");
        index.push_str("    C --> G[Quality]\n");
        index.push_str("    C --> H[Artifacts]\n");
        index.push_str("```\n\n");

        index.push_str("## Support\n\n");
        index.push_str("- **Documentation:** See individual guides for detailed information\n");
        index.push_str("- **Issues:** Report bugs and request features on GitHub\n");
        index.push_str("- **Discussions:** Join community discussions for questions\n\n");

        index.push_str("---\n\n");
        index.push_str("*This documentation was automatically generated by Agent Agency V3.*\n");

        Ok(index)
    }

    /// Save documentation to files
    pub async fn save_to_files(&self, docs: &HashMap<String, String>) -> Result<(), DocumentationError> {
        use tokio::fs;

        // Create output directory
        fs::create_dir_all(&self.config.output_directory).await
            .map_err(|e| DocumentationError::IoError(e))?;

        // Write each document
        for (filename, content) in docs {
            let path = self.config.output_directory.join(filename);
            fs::write(&path, content).await
                .map_err(|e| DocumentationError::IoError(e))?;

            tracing::info!("Generated documentation: {}", path.display());
        }

        Ok(())
    }
}

pub type Result<T> = std::result::Result<T, DocumentationError>;

#[derive(Debug, thiserror::Error)]
pub enum DocumentationError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Documentation generation failed: {0}")]
    GenerationError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

