# Agent Agency V3 Web Dashboard

A comprehensive, production-ready observability and research platform for analyzing and controlling agent systems in real-time.

## 🚀 Overview

The Agent Agency V3 Dashboard provides researchers and developers with powerful tools to:

- **🤖 Research Agent Behavior**: Deep inspection of agent performance, decision-making, and coordination patterns
- **📊 Monitor System Health**: Real-time observability with automated anomaly detection and alerting
- **🔬 Analyze Performance**: Advanced statistical analysis including trend detection, forecasting, and correlation analysis
- **💬 Control Operations**: Conversational interface for task guidance and agent management
- **🗄️ Inspect Data**: Safe, read-only database exploration with vector search capabilities
- **🔮 Predict Outcomes**: ML-based forecasting with confidence intervals and factor analysis

## 🏗️ Architecture

### Tech Stack

- **Framework**: Next.js 14.2.15 with App Router
- **Language**: TypeScript with strict type checking
- **Styling**: SCSS Modules (no external CSS frameworks)
- **State Management**: React hooks with proper error boundaries
- **Real-time Communication**: WebSocket + Server-Sent Events
- **API Layer**: RESTful clients with retry logic and structured errors
- **Testing**: Jest + React Testing Library infrastructure ready

### Project Structure

```
src/
├── app/                    # Next.js App Router pages
│   ├── api/               # API routes (proxy, health)
│   └── layout.tsx         # Root layout with providers
├── components/            # React components organized by domain
│   ├── analytics/         # Analytics & insights components
│   ├── chat/             # Conversational AI interface
│   ├── database/         # Database exploration tools
│   ├── metrics/          # System monitoring & KPIs
│   ├── monitoring/       # Real-time observability
│   ├── shared/           # Reusable UI components
│   └── tasks/            # Task monitoring & control
├── lib/                  # Business logic & external integrations
│   ├── analytics-api.ts  # Analytics API client
│   ├── api-client.ts     # Core HTTP client with retries
│   ├── chat-api.ts       # Chat/WebSocket API client
│   ├── database-api.ts   # Database query API client
│   └── websocket/        # WebSocket connection management
├── styles/               # Global styles & design system
│   ├── globals.scss      # CSS reset & typography
│   ├── mixins.scss       # Reusable SCSS mixins
│   └── variables.scss    # Design tokens & colors
└── types/                # TypeScript type definitions
    ├── analytics.ts      # Analytics domain types
    ├── chat.ts          # Chat/WebSocket types
    ├── database.ts      # Database schema types
    ├── metrics.ts       # Metrics & monitoring types
    └── tasks.ts         # Task management types
```

## 🎯 Core Features

### 1. 🤖 Agent Research Interface

**Deep inspection of agent behavior and performance:**

- Real-time agent monitoring with live metrics streaming
- Task execution visualization with progress tracking
- Agent coordination analysis and bottleneck detection
- Model performance comparison across different agents
- Self-prompting iteration analysis and improvement tracking

### 2. 📊 System Observability

**Comprehensive monitoring and alerting:**

- Real-time KPI dashboards with trend indicators
- Automated anomaly detection using statistical algorithms
- System health scoring with component-level monitoring
- Business intelligence metrics and conversion tracking
- Alert management with configurable thresholds

### 3. 🔬 Advanced Analytics

**Statistical analysis and predictive insights:**

- **Anomaly Detection**: Z-score, Isolation Forest algorithms for outlier identification
- **Trend Analysis**: Linear regression with seasonal decomposition
- **Performance Prediction**: Time series forecasting with confidence intervals
- **Correlation Analysis**: Statistical relationships between metrics with significance testing
- **Interactive Visualizations**: Charts with real-time data updates

### 4. 💬 Conversational Control

**Natural language interface for agent management:**

- WebSocket-powered real-time chat with agents
- Context-aware task guidance and instruction
- Session management with conversation history
- Multi-agent coordination through natural language commands
- Error handling and connection recovery

### 5. 🗄️ Database Exploration

**Safe, read-only database inspection:**

- Schema browser with table relationships and constraints
- SQL Query Builder with syntax highlighting and parameter support
- Vector similarity search for embedding analysis
- Data quality assessment with automated metrics
- Export functionality (CSV, JSON, SQL) with safety constraints

## 🚀 Quick Start

### Prerequisites

- Node.js 18.17.0 or higher
- npm or yarn package manager
- V3 backend server running (optional for development)

### Installation

```bash
# Clone the repository
git clone <repository-url>
cd agent-agency/iterations/v3/apps/web-dashboard

# Install dependencies
npm install

# Start development server
npm run dev
```

The dashboard will be available at `http://localhost:3005` (or the port specified by `WEB_DASHBOARD_PORT` environment variable).

### Environment Configuration

Create a `.env.local` file in the project root:

```bash
# V3 Backend Configuration
V3_BACKEND_HOST=http://localhost:8080

# Dashboard Configuration
WEB_DASHBOARD_PORT=3005

# Development Options
NODE_ENV=development
```

## 📖 Usage Guide

### Navigation

The dashboard features a tabbed interface with 6 main sections:

1. **Overview**: System health dashboard with key metrics and alerts
2. **Chat**: Conversational interface for agent control and guidance
3. **Tasks**: Task monitoring with progress tracking and action controls
4. **Database**: Database exploration with query builder and vector search
5. **Metrics**: Comprehensive metrics dashboard with real-time updates
6. **Analytics**: Advanced analytics with anomaly detection and forecasting

### Key Workflows

#### Researching Agent Performance

1. Navigate to **Tasks** tab to monitor active agent executions
2. Use **Analytics** tab to analyze performance trends and detect anomalies
3. Switch to **Database** tab to inspect data generated by agents
4. Use **Chat** interface to provide guidance or modify agent behavior

#### System Health Monitoring

1. Start with **Overview** tab for high-level system status
2. Monitor **Metrics** tab for real-time KPI updates
3. Check **Analytics** for automated anomaly detection
4. Use alert notifications to identify issues proactively

#### Database Analysis

1. Browse database schema in **Database** tab
2. Use query builder for custom data analysis
3. Perform vector searches for embedding similarity
4. Export results for external analysis

## 🔧 Development

### Available Scripts

```bash
# Development
npm run dev              # Start development server
npm run build           # Production build
npm run start           # Production server
npm run lint            # Run ESLint
npm run typecheck       # TypeScript type checking

# Testing (infrastructure ready)
npm run test            # Run Jest tests
npm run test:watch      # Watch mode testing
npm run test:coverage   # Generate coverage reports
```

### Code Quality Standards

The codebase follows strict quality standards:

- **ESLint**: Configured with TypeScript and React rules
- **TypeScript**: Strict mode with no implicit any types
- **SCSS Modules**: Component-scoped styling
- **Pre-commit hooks**: Automated quality checks
- **Conventional commits**: Standardized commit messages

### Component Development

Components follow a consistent pattern:

```typescript
// Component structure
interface ComponentProps {
  // Define props with proper typing
}

export default function ComponentName({
  prop1,
  prop2,
  ...props
}: ComponentProps) {
  // Component logic with proper error handling
  return (
    <div className={styles.container}>
      {/* Component JSX */}
    </div>
  );
}
```

### API Integration

The dashboard communicates with the V3 backend through structured API clients:

```typescript
// Example API usage
import { apiClient } from "@/lib/api-client";

const response = await apiClient.request(endpoint, {
  method: "POST",
  body: JSON.stringify(data),
});
```

## 🔌 Backend Integration

### Current Status

The dashboard is designed to work with the V3 backend APIs. Currently, all API calls are mocked with TODO comments indicating where real backend integration should occur.

### Required Backend Endpoints

#### Core APIs
- `GET /api/v1/health` - System health check
- `POST /api/v1/chat/ws/:session_id` - WebSocket chat endpoint
- `GET /api/v1/tasks/*` - Task management and monitoring
- `GET /api/v1/metrics/stream` - Real-time metrics streaming

#### Analytics APIs
- `GET /api/v1/analytics/summary` - Analytics overview
- `GET /api/v1/analytics/anomalies` - Anomaly detection
- `GET /api/v1/analytics/trends` - Trend analysis
- `GET /api/v1/analytics/predictions` - Performance forecasting
- `GET /api/v1/analytics/correlations` - Correlation analysis

#### Database APIs
- `GET /api/v1/database/connections` - Database connections
- `GET /api/v1/database/tables` - Table schemas and data
- `POST /api/v1/database/query` - SQL query execution
- `POST /api/v1/database/vector/search` - Vector similarity search

### Integration Checklist

- [ ] Implement V3 backend health check endpoint
- [ ] Add WebSocket support for real-time chat
- [ ] Connect task monitoring APIs
- [ ] Integrate metrics streaming
- [ ] Enable database query functionality
- [ ] Implement analytics processing APIs
- [ ] Add authentication and authorization
- [ ] Configure CORS and security headers

## 🧪 Testing

Testing infrastructure is configured but tests are not yet implemented:

```bash
# Run tests (when implemented)
npm run test

# Generate coverage report
npm run test:coverage
```

### Testing Strategy

- **Unit Tests**: Individual component and utility function testing
- **Integration Tests**: API client and component interaction testing
- **E2E Tests**: Critical user journey testing with Playwright/Cypress
- **Accessibility Tests**: WCAG compliance verification

## 🚢 Deployment

### Docker Deployment

```dockerfile
FROM node:18-alpine
WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production
COPY . .
RUN npm run build
EXPOSE 3005
CMD ["npm", "start"]
```

### Environment Variables

```bash
# Production environment
NODE_ENV=production
V3_BACKEND_HOST=https://api.agent-agency.com
WEB_DASHBOARD_PORT=3005

# Security
NEXTAUTH_SECRET=your-secret-key
NEXTAUTH_URL=https://dashboard.agent-agency.com

# Monitoring
SENTRY_DSN=your-sentry-dsn
```

## 🤝 Contributing

### Development Workflow

1. **Fork and clone** the repository
2. **Create a feature branch** from `main`
3. **Make changes** following the established patterns
4. **Run quality checks**: `npm run lint && npm run typecheck`
5. **Test your changes** (when tests are implemented)
6. **Commit with conventional format** and create PR

### Code Standards

- Use TypeScript with strict mode
- Follow React best practices and hooks patterns
- Write self-documenting code with clear naming
- Add JSDoc comments for public APIs
- Use SCSS modules for component styling
- Follow conventional commit format

## 📊 Performance & Monitoring

### Performance Budgets

- **Bundle Size**: < 500KB gzipped
- **First Contentful Paint**: < 1.5s
- **Largest Contentful Paint**: < 2.5s
- **Time to Interactive**: < 3.5s

### Monitoring

- **Error Tracking**: Sentry integration ready
- **Performance Monitoring**: Web vitals tracking
- **User Analytics**: Usage pattern analysis
- **API Monitoring**: Request/response metrics

## 🔒 Security

### Security Measures

- **Input Validation**: All user inputs validated and sanitized
- **CORS Configuration**: Proper cross-origin request handling
- **Content Security Policy**: XSS protection headers
- **Authentication**: Secure session management
- **Data Encryption**: Sensitive data encryption at rest and in transit

### Security Checklist

- [ ] Input validation on all forms and API calls
- [ ] SQL injection prevention in query builder
- [ ] XSS protection with proper content escaping
- [ ] CSRF protection for state-changing operations
- [ ] Secure WebSocket connections
- [ ] Rate limiting on API endpoints
- [ ] Audit logging for sensitive operations

## 📚 Documentation

### Additional Documentation

- **API Reference**: Detailed API endpoint documentation
- **Component Library**: Reusable component documentation
- **Architecture Guide**: System design and data flow diagrams
- **Deployment Guide**: Production deployment procedures
- **Troubleshooting**: Common issues and solutions

## 🎯 Roadmap

### Phase 1: Core Completion ✅
- [x] Complete all major UI components
- [x] Implement comprehensive TypeScript types
- [x] Add proper error handling and loading states
- [x] Create responsive design system

### Phase 2: Backend Integration 🔄
- [ ] Implement V3 backend API connections
- [ ] Add real-time WebSocket support
- [ ] Enable database query functionality
- [ ] Connect analytics processing APIs

### Phase 3: Production Readiness 📋
- [ ] Implement comprehensive test suite
- [ ] Add performance monitoring and optimization
- [ ] Configure CI/CD pipeline
- [ ] Add security hardening and audit

### Phase 4: Advanced Features 🔮
- [ ] Add machine learning insights
- [ ] Implement advanced visualization
- [ ] Add collaborative features
- [ ] Enable plugin architecture

## 🆘 Support

### Getting Help

- **Documentation**: Check the docs folder for detailed guides
- **Issues**: Create GitHub issues for bugs and feature requests
- **Discussions**: Use GitHub discussions for questions and ideas

### Common Issues

**Dashboard not loading:**
- Check that V3 backend is running
- Verify environment variables are set correctly
- Check browser console for JavaScript errors

**API calls failing:**
- Ensure V3 backend endpoints are implemented
- Check network connectivity and CORS settings
- Verify authentication tokens if required

**Performance issues:**
- Check browser developer tools for bottlenecks
- Verify bundle size is within limits
- Check for memory leaks in React components

## 📄 License

[License information to be added]

---

**Agent Agency V3 Dashboard** - Transforming agent research through comprehensive observability and intelligent analysis. 🔬⚡🎯
