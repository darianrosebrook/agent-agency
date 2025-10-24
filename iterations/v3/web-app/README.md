# Agent Agency V3 Web Interface

A modern web interface for the Agent Agency V3 orchestration platform, providing an intuitive dashboard for managing AI tasks and monitoring system health.

## Features

- **Real-time Dashboard**: Monitor system health, worker status, and task execution
- **Task Management**: Create and execute AI tasks with configurable risk tiers
- **Worker Monitoring**: View registered workers and their capabilities
- **Live Logs**: Real-time system activity and task execution logs
- **Responsive Design**: Modern UI built with Tailwind CSS

## Architecture

The web interface communicates with the Agent Agency V3 API server via REST endpoints:

- `GET /api/health` - System health metrics
- `GET /api/tasks/active` - List active tasks
- `POST /api/tasks/{task_id}/execute` - Execute a new task
- `POST /api/workers/register` - Register a worker
- `POST /api/workers/{worker_id}/health` - Update worker health

## Getting Started

### Prerequisites

1. **Agent Agency V3 API Server**: Ensure the orchestration service is running
   ```bash
   cd ../..
   export DATABASE_URL="postgresql://user:password@localhost/agent_agency"
   cargo run --features api-server -- serve --port 8080 --database-url $DATABASE_URL
   ```

2. **Web Browser**: Modern browser with JavaScript enabled

### Running the Web Interface

```bash
# Install dependencies (optional, for development)
npm install

# Start the web server
npm run dev
# or
python3 -m http.server 3000

# Open in browser
open http://localhost:3000
```

## Usage

### Creating Tasks

1. Enter a task description in the text area
2. Select appropriate risk tier (1-3)
3. Configure scope (file paths the task can access)
4. Click "Execute Task" to submit

### Monitoring System

- **Dashboard Cards**: Real-time metrics for workers, tasks, and system health
- **Active Tasks**: View currently running tasks with status indicators
- **Worker Pool**: See registered workers and their capabilities
- **System Logs**: Live activity feed with timestamps and status colors

## Development

### Project Structure

```
web-app/
├── index.html          # Main application
├── package.json        # NPM configuration
├── README.md          # This file
└── server.py          # Alternative Python server
```

### API Integration

The interface uses the Fetch API for communication:

```javascript
// Example API call
const response = await fetch('http://localhost:8080/api/health');
const health = await response.json();
```

### Styling

Built with Tailwind CSS for responsive, modern design:
- Dark theme optimized for development environments
- Responsive grid layouts
- Consistent color scheme and typography

## Troubleshooting

### Connection Issues

**Problem**: "Connection Failed" status
**Solution**:
1. Verify API server is running on port 8080
2. Check CORS settings on API server
3. Ensure firewall allows local connections

### No Data Display

**Problem**: Dashboard shows "-" values
**Solution**:
1. Check API server logs for errors
2. Verify database connection
3. Ensure proper API endpoints are implemented

### Task Submission Fails

**Problem**: Tasks fail to submit
**Solution**:
1. Check browser console for JavaScript errors
2. Verify task data format matches API expectations
3. Check API server logs for validation errors

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly
5. Submit a pull request

## License

MIT License - see LICENSE file for details
