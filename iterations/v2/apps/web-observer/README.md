# Arbiter Observer Web Interface

A Next.js web interface for human users to interact with the Arbiter Observer system, providing the same functionality as the MCP client but through a visual web interface.

## Features

- **System Dashboard**: Real-time system status, metrics, and progress monitoring
- **Task Management**: Submit tasks, view task progress, and monitor chain-of-thought
- **Event Streaming**: Live event log viewer with filtering and real-time updates
- **Arbiter Control**: Start/stop the arbiter orchestrator and execute management commands
- **Observation Interface**: Add notes and observations to tasks

## Getting Started

### Prerequisites

- Node.js 18+
- The main Arbiter application running with the Observer HTTP server

### Installation

```bash
cd apps/web-observer
npm install
```

### Development

```bash
npm run dev
```

The web interface will be available at http://localhost:3000 (or the port specified by WEB_OBSERVER_PORT environment variable)

### Production Build

```bash
npm run build
npm start
```

## Architecture

The web interface connects to the Arbiter Observer HTTP API endpoints:

- **Status & Metrics**: `/observer/status`, `/observer/metrics`, `/observer/progress`
- **Tasks**: `/observer/tasks`, `/observer/tasks/{id}`, `/observer/tasks/{id}/cot`
- **Events**: `/observer/logs`, `/observer/events/stream`
- **Chain of Thought**: `/observer/cot`, `/observer/tasks/{id}/cot`
- **Arbiter Control**: `/observer/arbiter/start`, `/observer/arbiter/stop`, `/observer/commands`
- **Observations**: `/observer/observations`

## Configuration

### Port Configuration

The web interface port can be configured using the `WEB_OBSERVER_PORT` environment variable:

```bash
# Use port 3001 instead of default 3000
WEB_OBSERVER_PORT=3001 npm run dev
```

### Observer API Configuration

The interface automatically connects to the observer API at `http://127.0.0.1:4387` by default. This can be configured by setting the `OBSERVER_URL` environment variable in the main application.

## Features Overview

### Dashboard

- Real-time system status and health monitoring
- Performance metrics and reasoning progress
- Task success rates and system utilization

### Task Management

- Submit new tasks with descriptions and optional spec files
- View active tasks with progress indicators
- Detailed task view with chain-of-thought streaming
- Add observations and notes to tasks

### Event Viewer

- Real-time event log streaming
- Filter by severity, type, task ID, and time range
- Paginated event history
- Auto-refresh capability

### Arbiter Controls

- Start/stop the arbiter orchestrator
- Execute management commands
- Quick action buttons for common operations

## Integration with Main Application

The web interface is automatically started when the main Arbiter application runs. It integrates with the existing Observer Bridge and HTTP server infrastructure.

To start both the main application and web interface:

```bash
cd iterations/v2
npm run dev  # This will start the main app and web interface
```

The web interface will be available at http://localhost:3000 (or the port specified by WEB_OBSERVER_PORT environment variable) while the main application runs.
