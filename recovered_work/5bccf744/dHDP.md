# Agent Agency V3 Dashboard

A modern web dashboard for monitoring and managing agent task execution in the Agent Agency V3 system.

## Features

- **Real-time Metrics Visualization**: Live system metrics from V3 backend (CPU, memory, tasks, performance)
- **Real-time Task Monitoring**: View task status, progress, and execution details with live updates
- **Interactive Task Management**: Pause, resume, cancel, and retry tasks
- **Comprehensive Audit Trail**: Track all task actions and state changes
- **System Health Dashboard**: Real-time component status monitoring (API, database, orchestrator, workers)
- **Connection Status Monitoring**: Live V3 backend connectivity status in header
- **Agent Performance Tracking**: Real-time agent metrics and coordination efficiency
- **Business Intelligence**: Error rates, throughput metrics, and trend analysis
- **Alert Management System**: Automated notifications, escalation policies, and incident response
- **Real-time Alert Dashboard**: Live alert monitoring with acknowledge/resolve actions
- **Alert Statistics**: Comprehensive metrics on alert patterns and resolution times
- **Responsive Design**: Works on desktop, tablet, and mobile devices
- **Dark Mode Support**: Automatic theme switching based on system preferences

## Technology Stack

- **Framework**: Next.js 14 with App Router
- **Language**: TypeScript
- **Styling**: SCSS with CSS Modules
- **State Management**: React Hooks
- **API Client**: Custom HTTP client with error handling
- **Real-time Updates**: Server-Sent Events (SSE) from V3 backend for live metrics
- **Alerting System**: Automated notification channels (Email, Slack, PagerDuty, Webhook)
- **Data Visualization**: Custom React components with real-time trend analysis

## Getting Started

### Prerequisites

- Node.js 18.0.0 or higher
- npm 8.0.0 or higher
- Agent Agency V3 backend running on `http://localhost:8080`

### Installation

1. Clone the repository and navigate to the dashboard directory:
   ```bash
   cd iterations/v3/apps/web-dashboard
   ```

2. Install dependencies:
   ```bash
   npm install
   ```

3. Test V3 backend connectivity (recommended):
   ```bash
   node test-connection.js
   ```

4. Set up environment variables:
   ```bash
   cp .env.example .env.local
   ```
   
   Edit `.env.local` and configure:
   ```env
   V3_BACKEND_HOST=http://localhost:8080
   NEXT_PUBLIC_API_BASE_URL=/api/proxy/v1
   ```

4. Start the development server:
   ```bash
   npm run dev
   ```

5. Open [http://localhost:3000](http://localhost:3000) in your browser.

## Real-time Metrics Visualization

The dashboard provides comprehensive real-time monitoring of the Agent Agency V3 system:

### Live System Metrics
- **CPU & Memory Usage**: Real-time system resource monitoring
- **Task Statistics**: Active, completed, and failed task counts
- **Response Times**: Average API response times
- **Component Health**: API, database, orchestrator, and worker status

### Real-time Updates
- **Server-Sent Events (SSE)**: Live streaming from V3 backend
- **Connection Status**: Header indicator shows V3 backend connectivity
- **Automatic Reconnection**: Handles connection drops gracefully
- **Trend Analysis**: Historical KPI tracking with trend indicators

### Metrics Dashboard Features
- **Key Performance Indicators**: System health, active agents, throughput, response times
- **System Resources Monitor**: Visual progress bars for CPU/memory usage
- **Agent Performance**: Real-time agent metrics and coordination efficiency
- **Business Intelligence**: Error rates and throughput analysis
- **Historical Trends**: Rolling 10-value trend calculations

### Connection Testing
Before starting the dashboard, verify V3 backend connectivity:

```bash
# Test all endpoints and real-time streaming
node test-connection.js

# Expected output:
✅ Health endpoint responding
✅ Metrics endpoint responding
✅ Metrics stream connected
✅ Received real-time metrics data
📈 CPU: 25.3%
🧠 Memory: 45.7%
⚙️ Active Tasks: 3
```

### Environment Configuration
```env
# Required for real-time metrics
V3_BACKEND_HOST=http://localhost:8080

# Optional: Connection settings
# V3_CONNECT_TIMEOUT=5000
# V3_RECONNECT_INTERVAL=30000
```

## Alerting System

The dashboard includes a comprehensive alerting system for automated failure notifications and incident response.

### Alert Types

The system monitors for various types of issues:

- **System Health**: CPU usage, memory usage, service availability
- **Performance**: Response times, error rates, throughput
- **Security**: Authentication failures, suspicious activity
- **Availability**: Service downtime, connectivity issues
- **Compliance**: RTO/RPO violations, SLA breaches

### Alert Severities

- **Critical** 🚨: Immediate action required (service down, data loss)
- **Error** ❌: System errors requiring attention
- **Warning** ⚠️: Potential issues that should be monitored
- **Info** ℹ️: Informational alerts for awareness

### Notification Channels

Alerts are sent through multiple channels:

- **Email**: Admin notifications with detailed information
- **Slack**: Real-time team notifications in dedicated channels
- **PagerDuty**: Critical alerts with escalation policies
- **Webhook**: Integration with external monitoring systems
- **SMS**: Critical alerts for on-call personnel

### Alert Management

The Alert Dashboard provides:

- **Real-time Alert Feed**: Live stream of all active alerts
- **Alert Filtering**: Filter by severity, status, and category
- **Alert Actions**: Acknowledge and resolve alerts directly from UI
- **Alert Statistics**: Metrics on alert patterns and resolution times
- **Escalation Tracking**: Monitor alert escalation levels

### Alert Workflow

1. **Detection**: System monitors trigger alerts based on defined conditions
2. **Notification**: Alerts are sent through configured channels
3. **Escalation**: Unacknowledged alerts escalate based on policies
4. **Acknowledgment**: Team members acknowledge alerts they're working on
5. **Resolution**: Alerts are resolved when issues are fixed
6. **Review**: Alert history and patterns are analyzed for improvement

### Configuration

Alerts are configured in the V3 backend with:

- **Alert Definitions**: Conditions, thresholds, and notification settings
- **Escalation Policies**: When and how alerts should escalate
- **Notification Channels**: Where alerts should be sent
- **Suppression Rules**: When alerts should be temporarily disabled

### Example Alert Configuration

```json
{
  "id": "cpu_high_usage",
  "name": "High CPU Usage",
  "severity": "warning",
  "condition": {
    "metric": "cpu_usage_percent",
    "operator": "greater_than",
    "threshold": 80,
    "duration_secs": 300
  },
  "channels": ["slack-alerts", "email-admin"],
  "escalation_policy": "default-escalation"
}
```

### Alert API Endpoints

The dashboard provides REST API endpoints for alert management:

- `GET /api/alerts` - List active alerts
- `POST /api/alerts/{id}/acknowledge` - Acknowledge an alert
- `POST /api/alerts/{id}/resolve` - Resolve an alert
- `GET /api/alerts/statistics` - Get alert statistics

### Integration with Monitoring

The alerting system integrates with:

- **Real-time Metrics**: Alerts are triggered based on live system metrics
- **System Health**: Component status monitoring
- **Performance Monitoring**: Response time and throughput tracking
- **Error Tracking**: Automatic error rate monitoring

## Project Structure

```
src/
├── app/                    # Next.js App Router pages
│   ├── globals.scss       # Global styles and CSS variables
│   ├── layout.tsx         # Root layout component
│   ├── page.tsx           # Dashboard home page
│   ├── tasks/             # Task-related pages
│   │   ├── page.tsx       # Task list page
│   │   └── [taskId]/      # Task detail page
│   └── api/               # API routes (proxy to backend)
├── components/            # Reusable React components
│   ├── shared/           # Shared components (Header, Navigation)
│   └── tasks/            # Task-specific components
├── lib/                  # Utility libraries
│   ├── api-client.ts     # Generic API client
│   └── task-api.ts       # Task-specific API client
└── types/                # TypeScript type definitions
    └── tasks.ts          # Task-related types
```

## API Integration

The dashboard communicates with the Agent Agency V3 backend through:

- **REST API**: Task management, metrics, and system status
- **WebSocket**: Real-time task updates and progress
- **Server-Sent Events**: Live metrics streaming

### API Endpoints

- `GET /api/v1/tasks` - List tasks with filtering
- `GET /api/v1/tasks/:id` - Get task details
- `POST /api/v1/tasks` - Submit new task
- `POST /api/v1/tasks/:id/pause` - Pause task
- `POST /api/v1/tasks/:id/resume` - Resume task
- `POST /api/v1/tasks/:id/cancel` - Cancel task
- `POST /api/v1/tasks/:id/retry` - Retry failed task
- `GET /api/v1/tasks/:id/audit` - Get task audit trail
- `GET /api/v1/metrics` - Get system metrics
- `GET /api/v1/metrics/stream` - Stream live metrics
- `WS /api/v1/tasks/:id/ws` - WebSocket for task updates

## Development

### Available Scripts

- `npm run dev` - Start development server
- `npm run build` - Build for production
- `npm run start` - Start production server
- `npm run lint` - Run ESLint
- `npm run lint:fix` - Fix ESLint errors
- `npm run type-check` - Run TypeScript type checking
- `npm run format` - Format code with Prettier
- `npm run format:check` - Check code formatting
- `npm run test` - Run tests
- `npm run test:watch` - Run tests in watch mode
- `npm run test:coverage` - Run tests with coverage
- `npm run clean` - Clean build artifacts

### Code Style

The project uses:

- **ESLint** for code linting
- **Prettier** for code formatting
- **TypeScript** for type safety
- **SCSS** for styling with CSS Modules

### Component Guidelines

- Use functional components with React Hooks
- Implement proper TypeScript types
- Follow the established naming conventions
- Use CSS Modules for component styling
- Implement responsive design patterns

## Deployment

### Production Build

1. Build the application:
   ```bash
   npm run build
   ```

2. Start the production server:
   ```bash
   npm run start
   ```

### Docker Deployment

The dashboard can be deployed using Docker:

```dockerfile
FROM node:18-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production
COPY . .
RUN npm run build

FROM node:18-alpine AS runner
WORKDIR /app
COPY --from=builder /app/.next ./.next
COPY --from=builder /app/public ./public
COPY --from=builder /app/package*.json ./
RUN npm ci --only=production
EXPOSE 3000
CMD ["npm", "start"]
```

## Configuration

### Environment Variables

- `V3_BACKEND_HOST` - Backend API host URL
- `NEXT_PUBLIC_API_BASE_URL` - Public API base URL
- `NODE_ENV` - Environment (development, production)

### Customization

The dashboard can be customized through:

- **CSS Variables**: Modify colors, spacing, and typography
- **Component Props**: Customize component behavior
- **API Configuration**: Adjust API endpoints and timeouts
- **Theme Settings**: Configure light/dark mode preferences

## Troubleshooting

### Common Issues

1. **API Connection Errors**
   - Verify the backend is running on the correct port
   - Check the `V3_BACKEND_HOST` environment variable
   - Ensure CORS is properly configured

2. **Build Errors**
   - Run `npm run type-check` to identify TypeScript errors
   - Check for missing dependencies
   - Verify all imports are correct

3. **Styling Issues**
   - Ensure SCSS files are properly imported
   - Check CSS Module class names
   - Verify responsive breakpoints

### Debug Mode

Enable debug logging by setting:
```env
NODE_ENV=development
DEBUG=agent-agency:*
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests and linting
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Support

For support and questions:

- Create an issue in the repository
- Check the documentation
- Review the API reference
- Contact the development team