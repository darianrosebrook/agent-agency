# Technical Specification & Implementation Details

## Ambiguities Analysis

### ✅ Resolved Ambiguities

1. **API Contract Definitions**: All major endpoints specified with HTTP methods and paths
2. **Data Flow Patterns**: Real-time updates via WebSocket/SSE clearly defined
3. **Authentication Model**: Role-based access control with specific permission levels
4. **Component Architecture**: UI component breakdown with clear responsibilities
5. **State Management**: Centralized state patterns with specific store structures

### ⚠️ Remaining Ambiguities

1. **Exact API Response Formats**: JSON schemas not fully specified for complex responses
2. **Error Handling**: Specific error codes and user-facing messages not detailed
3. **Pagination Strategies**: Cursor vs offset pagination not specified per endpoint
4. **Caching Policies**: TTL and invalidation strategies not defined
5. **Rate Limiting**: API rate limits and user feedback not specified

## Bill of Materials

### Core Infrastructure Components

#### **API Client Layer**
```typescript
// Unified API client with error handling and retries
class ApiClient {
  private baseUrl: string;
  private authToken: string;
  private retryConfig: RetryConfig;

  async request<T>(endpoint: string, options: RequestOptions): Promise<T>
  async stream<T>(endpoint: string, callback: (data: T) => void): Promise<void>
}
```

#### **Real-time Communication**
```typescript
// WebSocket connection manager
class WebSocketManager {
  private connections: Map<string, WebSocket>;
  private reconnectStrategy: ReconnectStrategy;

  connect(channel: string): Promise<WebSocket>
  disconnect(channel: string): void
  subscribe<T>(channel: string, handler: (data: T) => void): void
}

// SSE client for server-sent events
class SSEClient {
  private eventSource: EventSource;
  private reconnectTimer: NodeJS.Timeout;

  connect(endpoint: string): Promise<EventSource>
  onMessage<T>(handler: (data: T) => void): void
}
```

### UI Component Inventory

#### **Shared Components** (Reusable)
```typescript
// Core design system components
interface ButtonProps extends BaseProps {
  variant: 'primary' | 'secondary' | 'danger';
  size: 'sm' | 'md' | 'lg';
  loading?: boolean;
}

interface MetricCardProps {
  title: string;
  value: string | number;
  trend?: TrendIndicator;
  icon?: React.ReactNode;
}

interface DataTableProps<T> {
  data: T[];
  columns: ColumnDefinition<T>[];
  pagination?: PaginationConfig;
  sorting?: SortingConfig;
  filtering?: FilterConfig;
}

// Status and feedback components
interface StatusBadgeProps {
  status: 'success' | 'warning' | 'error' | 'info';
  text: string;
  animated?: boolean;
}

interface LoadingSkeletonProps {
  variant: 'card' | 'table' | 'chart';
  lines?: number;
  width?: string | number;
}
```

#### **Feature-Specific Components**

**Council Oversight:**
- `VerdictCard`, `VerdictDetailModal`, `EthicalConcernPanel`
- `JudgeMetricsDashboard`, `DecisionFlowDiagram`, `InterventionForm`

**Apple Silicon:**
- `HardwareMetricsGrid`, `ThermalStatusPanel`, `ModelPerformanceCharts`
- `RoutingVisualizer`, `OptimizationRecommendations`

**Security:**
- `AuthMonitoringDashboard`, `AccessControlPanel`, `SecretsInventory`
- `ThreatDetectionInterface`, `IncidentResponseWorkflow`

**Workspace:**
- `WorkspaceHealthDashboard`, `StateManager`, `GitOperationsInterface`
- `BackupRecoveryPanel`, `StateComparisonViewer`

### API Endpoint Inventory

#### **Council APIs**
```typescript
// Verdict management
GET  /api/council/verdicts?status=pending&limit=50
GET  /api/council/verdicts/{id}
POST /api/council/verdicts/{id}/override
GET  /api/council/verdicts/{id}/evidence

// Judge management
GET  /api/council/judges
GET  /api/council/judges/{id}/performance
GET  /api/council/judges/metrics

// Ethical assessment
GET  /api/council/ethical/assessments
POST /api/council/ethical/assessments/{id}/review
```

#### **Apple Silicon APIs**
```typescript
// Hardware metrics
GET  /api/apple-silicon/metrics/current
GET  /api/apple-silicon/metrics/history?period=1h&resolution=1m
GET  /api/apple-silicon/models/status

// Thermal management
GET  /api/apple-silicon/thermal/status
POST /api/apple-silicon/thermal/policy
GET  /api/apple-silicon/thermal/throttling/events

// Model routing
GET  /api/apple-silicon/routing/decisions
GET  /api/apple-silicon/routing/load-balance
POST /api/apple-silicon/routing/override
```

#### **Security APIs**
```typescript
// Authentication monitoring
GET  /api/security/auth/events?period=24h
GET  /api/security/auth/sessions/active
POST /api/security/auth/sessions/{id}/terminate

// Access control
GET  /api/security/access/roles
POST /api/security/access/roles
GET  /api/security/access/permissions/{userId}

// Secrets management
GET  /api/security/secrets
POST /api/security/secrets
POST /api/security/secrets/{id}/rotate

// Threat detection
GET  /api/security/threats/alerts
POST /api/security/threats/alerts/{id}/acknowledge
GET  /api/security/threats/incidents
```

#### **Workspace APIs**
```typescript
// Health monitoring
GET  /api/workspace/health/status
GET  /api/workspace/health/checks
POST /api/workspace/health/repair

// State management
POST /api/workspace/state/capture
GET  /api/workspace/state/snapshots
GET  /api/workspace/state/compare?from={id}&to={id}

// Git operations
GET  /api/workspace/git/status
POST /api/workspace/git/commit
GET  /api/workspace/git/history
POST /api/workspace/git/merge

// Backup/Recovery
GET  /api/workspace/backup/status
POST /api/workspace/backup/create
GET  /api/workspace/recovery/points
POST /api/workspace/recovery/restore
```

## Key Functions & Pseudocode

### Core State Management

```typescript
// Global state reducer pattern
interface DashboardState {
  user: UserProfile;
  council: CouncilState;
  appleSilicon: AppleSiliconState;
  security: SecurityState;
  workspace: WorkspaceState;
  ui: UIState;
}

function dashboardReducer(state: DashboardState, action: DashboardAction): DashboardState {
  switch (action.type) {
    case 'COUNCIL_VERDICT_RECEIVED':
      return {
        ...state,
        council: {
          ...state.council,
          verdicts: [action.payload, ...state.council.verdicts]
        }
      };

    case 'APPLE_SILICON_METRICS_UPDATE':
      return {
        ...state,
        appleSilicon: {
          ...state.appleSilicon,
          metrics: mergeMetrics(state.appleSilicon.metrics, action.payload)
        }
      };

    default:
      return state;
  }
}
```

### Real-time Data Synchronization

```typescript
// WebSocket connection management
class RealTimeManager {
  private connections = new Map<string, WebSocket>();
  private subscribers = new Map<string, Set<(data: any) => void>>();

  connect(channel: string): Promise<WebSocket> {
    return new Promise((resolve, reject) => {
      const ws = new WebSocket(`${WS_BASE_URL}/${channel}`);

      ws.onopen = () => {
        this.connections.set(channel, ws);
        resolve(ws);
      };

      ws.onmessage = (event) => {
        const data = JSON.parse(event.data);
        this.notifySubscribers(channel, data);
      };

      ws.onclose = () => {
        this.connections.delete(channel);
        // Auto-reconnect logic
        setTimeout(() => this.connect(channel), RECONNECT_DELAY);
      };

      ws.onerror = reject;
    });
  }

  subscribe<T>(channel: string, handler: (data: T) => void): () => void {
    if (!this.subscribers.has(channel)) {
      this.subscribers.set(channel, new Set());
    }
    this.subscribers.get(channel)!.add(handler);

    return () => {
      this.subscribers.get(channel)?.delete(handler);
    };
  }

  private notifySubscribers(channel: string, data: any) {
    const handlers = this.subscribers.get(channel);
    if (handlers) {
      handlers.forEach(handler => handler(data));
    }
  }
}
```

### Data Fetching with Error Handling

```typescript
// API client with comprehensive error handling
class ApiClient {
  private baseUrl: string;
  private authToken: string;
  private retryConfig = {
    maxRetries: 3,
    baseDelay: 1000,
    maxDelay: 10000
  };

  async request<T>(
    endpoint: string,
    options: RequestOptions = {}
  ): Promise<ApiResponse<T>> {
    const url = `${this.baseUrl}${endpoint}`;
    const headers = {
      'Authorization': `Bearer ${this.authToken}`,
      'Content-Type': 'application/json',
      ...options.headers
    };

    let attempt = 0;
    while (attempt <= this.retryConfig.maxRetries) {
      try {
        const response = await fetch(url, {
          ...options,
          headers
        });

        if (!response.ok) {
          throw new ApiError(response.status, await response.text());
        }

        const data = await response.json();
        return { success: true, data };

      } catch (error) {
        attempt++;

        if (attempt > this.retryConfig.maxRetries || !this.isRetryableError(error)) {
          return {
            success: false,
            error: this.normalizeError(error)
          };
        }

        // Exponential backoff
        const delay = Math.min(
          this.retryConfig.baseDelay * Math.pow(2, attempt - 1),
          this.retryConfig.maxDelay
        );
        await new Promise(resolve => setTimeout(resolve, delay));
      }
    }
  }

  private isRetryableError(error: any): boolean {
    // Network errors, 5xx server errors, specific 4xx errors
    return error.name === 'NetworkError' ||
           (error.status >= 500) ||
           error.status === 429;
  }

  private normalizeError(error: any): ApiError {
    if (error instanceof ApiError) return error;

    return new ApiError(
      error.status || 0,
      error.message || 'Unknown error occurred'
    );
  }
}
```

### Component State Management

```typescript
// Custom hook for council oversight
function useCouncilData() {
  const [state, dispatch] = useReducer(councilReducer, initialCouncilState);

  // Real-time verdict updates
  useEffect(() => {
    const unsubscribe = realTimeManager.subscribe< Verdict >(
      'council/verdicts',
      (verdict) => {
        dispatch({ type: 'VERDICT_RECEIVED', payload: verdict });
      }
    );

    return unsubscribe;
  }, []);

  // Initial data load
  useEffect(() => {
    apiClient.request<Verdict[]>('/api/council/verdicts')
      .then(result => {
        if (result.success) {
          dispatch({ type: 'VERDICTS_LOADED', payload: result.data });
        }
      });
  }, []);

  const interveneInVerdict = useCallback(async (verdictId: string, decision: InterventionDecision) => {
    dispatch({ type: 'INTERVENTION_STARTED', payload: verdictId });

    const result = await apiClient.request(
      `/api/council/verdicts/${verdictId}/override`,
      {
        method: 'POST',
        body: JSON.stringify(decision)
      }
    );

    if (result.success) {
      dispatch({ type: 'INTERVENTION_SUCCESS', payload: result.data });
    } else {
      dispatch({ type: 'INTERVENTION_FAILED', payload: result.error });
    }
  }, []);

  return {
    ...state,
    interveneInVerdict
  };
}
```

## Dependencies Analysis

### Core Dependencies (Required)

#### **React Ecosystem**
```json
{
  "next": "^14.0.0",
  "react": "^18.2.0",
  "react-dom": "^18.2.0",
  "@types/react": "^18.2.0",
  "@types/react-dom": "^18.2.0"
}
```

#### **State Management & Data Fetching**
```json
{
  "zustand": "^4.4.0",           // Lightweight state management
  "swr": "^2.2.0",               // Data fetching and caching
  "@tanstack/react-query": "^5.0.0", // Alternative data fetching
  "axios": "^1.6.0"              // HTTP client
}
```

#### **Real-time Communication**
```json
{
  "socket.io-client": "^4.7.0",  // WebSocket client
  "eventsource": "^2.0.2"        // SSE client
}
```

#### **UI Components & Styling**
```json
{
  "lucide-react": "^0.294.0",    // Icon library
  "framer-motion": "^10.16.0",   // Animation library
  "gsap": "^3.12.0",             // Advanced animations
  "tailwindcss": "^3.3.0",       // Utility-first CSS
  "sass": "^1.69.0",             // SCSS support
  "clsx": "^2.0.0",              // Conditional classes
  "tailwind-merge": "^2.0.0"     // Tailwind class merging
}
```

#### **Charts & Visualization**
```json
{
  "recharts": "^2.8.0",          // React charting library
  "d3": "^7.8.0",                // Data visualization
  "vis-network": "^9.1.0",       // Network/graph visualization
  "react-flow": "^11.8.0",       // Flow diagrams
  "mermaid": "^10.6.0"           // Diagram rendering
}
```

#### **Forms & Validation**
```json
{
  "react-hook-form": "^7.48.0",  // Form management
  "zod": "^3.22.0",              // Schema validation
  "yup": "^1.3.0",               // Alternative validation
  "@hookform/resolvers": "^3.3.0" // Validation resolvers
}
```

### Specialized Dependencies (Feature-Specific)

#### **Council Oversight**
```json
{
  "react-flow": "^11.8.0",       // Decision flow diagrams
  "vis-network": "^9.1.0",       // Judge relationship graphs
  "mermaid": "^10.6.0"           // Flowchart rendering
}
```

#### **Apple Silicon Monitoring**
```json
{
  "recharts": "^2.8.0",          // Performance charts
  "d3": "^7.8.0",                // Hardware heatmaps
  "three.js": "^0.158.0",        // 3D hardware visualization (optional)
  "@react-three/fiber": "^8.15.0" // React Three.js renderer
}
```

#### **Security Dashboard**
```json
{
  "react-syntax-highlighter": "^15.5.0", // Code/log highlighting
  "react-json-view": "^1.21.3",  // JSON data viewer
  "react-table": "^7.8.0",       // Advanced data tables
  "react-window": "^1.8.9"       // Virtual scrolling
}
```

#### **Workspace Management**
```json
{
  "react-diff-viewer": "^3.1.1",  // File diff visualization
  "react-monaco-editor": "^0.52.0", // Code editor for diffs
  "react-treebeard": "^3.3.0",   // File tree component
  "react-gitgraph": "^1.3.0"     // Git history visualization
}
```

### Development Dependencies

#### **Testing**
```json
{
  "@testing-library/react": "^14.1.0",
  "@testing-library/jest-dom": "^6.1.0",
  "@testing-library/user-event": "^14.5.0",
  "jest": "^29.7.0",
  "jest-environment-jsdom": "^29.7.0",
  "msw": "^1.3.0",               // API mocking
  "cypress": "^13.6.0"           // E2E testing
}
```

#### **Code Quality**
```json
{
  "eslint": "^8.55.0",
  "@typescript-eslint/eslint-plugin": "^6.14.0",
  "@typescript-eslint/parser": "^6.14.0",
  "prettier": "^3.1.0",
  "husky": "^8.0.0",             // Git hooks
  "lint-staged": "^15.2.0",      // Pre-commit linting
  "commitlint": "^18.4.0"        // Commit message linting
}
```

#### **Build & Development**
```json
{
  "typescript": "^5.3.0",
  "@types/node": "^20.10.0",
  "webpack": "^5.89.0",
  "webpack-bundle-analyzer": "^4.9.0",
  "next-compose-plugins": "^2.2.1"
}
```

### External Service Dependencies

#### **Backend Services**
- Agent Agency V3 API Server (Rust/Axum)
- Council Service (Decision making)
- Apple Silicon Service (Hardware monitoring)
- Database Service (PostgreSQL with extensions)
- Authentication Service (JWT/OAuth)

#### **External Integrations**
- **Grafana**: Dashboard embedding and alerting
- **Prometheus**: Metrics collection and querying
- **Git**: Repository operations (via system calls)
- **Email/SMS**: Alert notifications
- **SIEM**: Security event forwarding

#### **Cloud Services (Optional)**
- **AWS S3**: Backup storage
- **Redis**: Caching and session storage
- **PostgreSQL**: Primary data storage
- **Elasticsearch**: Advanced search and analytics

## Performance Optimization Strategy

### Bundle Optimization
- **Code Splitting**: Route-based and component-based splitting
- **Lazy Loading**: Dynamic imports for heavy components
- **Tree Shaking**: Remove unused code automatically
- **Compression**: Gzip/Brotli compression for assets

### Runtime Optimization
- **Virtual Scrolling**: For large data tables
- **Memoization**: React.memo and useMemo for expensive operations
- **Debouncing**: Input debouncing for search/filter operations
- **Caching**: SWR/React Query for API response caching

### Monitoring & Analytics
- **Performance Monitoring**: Web vitals tracking
- **Error Tracking**: Sentry integration
- **User Analytics**: Privacy-focused usage tracking
- **A/B Testing**: Feature flag system for testing

## Security Implementation

### Authentication & Authorization
```typescript
// JWT token management
class AuthManager {
  private token: string | null = null;
  private refreshToken: string | null = null;

  async login(credentials: LoginCredentials): Promise<AuthResult> {
    const response = await apiClient.request('/api/auth/login', {
      method: 'POST',
      body: JSON.stringify(credentials)
    });

    if (response.success) {
      this.setTokens(response.data);
      return { success: true };
    }

    return { success: false, error: response.error };
  }

  async refreshToken(): Promise<boolean> {
    if (!this.refreshToken) return false;

    const response = await apiClient.request('/api/auth/refresh', {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${this.refreshToken}`
      }
    });

    if (response.success) {
      this.setTokens(response.data);
      return true;
    }

    return false;
  }

  private setTokens(data: AuthTokens) {
    this.token = data.accessToken;
    this.refreshToken = data.refreshToken;
    localStorage.setItem('auth_token', this.token);
    localStorage.setItem('refresh_token', this.refreshToken);
  }
}
```

### Data Protection
- **Input Sanitization**: XSS prevention with DOMPurify
- **CSRF Protection**: CSRF tokens for state-changing operations
- **Content Security Policy**: Strict CSP headers
- **Secure Headers**: HSTS, X-Frame-Options, etc.

## Conclusion

This technical specification provides:
- ✅ **Resolved Ambiguities**: Clear API contracts, data flows, and component responsibilities
- ✅ **Complete Bill of Materials**: All UI components, API endpoints, and infrastructure components
- ✅ **Key Functions & Pseudocode**: Core algorithms for state management, real-time sync, and API communication
- ✅ **Dependencies Analysis**: Comprehensive list of required packages and external services

The specification is ready for implementation with all major technical decisions documented and all critical components identified.
