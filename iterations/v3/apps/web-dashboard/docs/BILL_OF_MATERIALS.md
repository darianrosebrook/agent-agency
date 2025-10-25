# Dashboard Bill of Materials (BOM)

## Overview

This document provides the complete bill of materials for the Agent Agency V3 web dashboard, including all components, dependencies, APIs, and technical specifications required for implementation.

## Current Dependencies (package.json)

### Core Framework
```json
{
  "next": "^16.0.0",
  "react": "^19.0.0",
  "react-dom": "^19.0.0",
  "typescript": "^5.0.0"
}
```

### UI & Styling
```json
{
  "sass": "^1.69.0",
  "tailwind-merge": "^3.3.1",
  "class-variance-authority": "^0.7.0",
  "clsx": "^2.1.1",
  "lucide-react": "^0.548.0"
}
```

### State Management & Data
```json
{
  "zustand": "^5.0.8"
}
```

### Animations
```json
{
  "gsap": "^3.13.0"
}
```

## Additional Dependencies Required

### Dashboard-Specific Dependencies

#### Real-time Communication
```json
{
  "@stomp/stompjs": "^7.0.0",           // WebSocket/SockJS client
  "socket.io-client": "^4.7.0",         // Socket.io client
  "reconnecting-websocket": "^4.4.0",  // Auto-reconnecting WebSocket
  "event-source-polyfill": "^1.0.31"   // SSE polyfill for older browsers
}
```

#### Data Visualization & Charts
```json
{
  "recharts": "^2.13.0",                // React charting library
  "d3": "^7.9.0",                       // D3.js for advanced visualizations
  "@visx/visx": "^3.12.0",              // Airbnb's visualization library
  "react-flow": "^11.11.0",             // Flow diagrams for decision trees
  "react-vis": "^1.11.7",               // React visualization components
  "victory": "^36.9.0"                  // Declarative charting library
}
```

#### UI Component Libraries
```json
{
  "@headlessui/react": "^2.2.0",         // Headless UI components
  "@heroicons/react": "^2.1.0",          // Heroicons for consistent icons
  "react-select": "^5.8.0",             // Advanced select components
  "react-datepicker": "^7.4.0",         // Date picker components
  "react-table": "^7.8.0",              // Table components with sorting/filtering
  "@tanstack/react-table": "^8.20.0",   // Modern table library
  "react-virtual": "^2.10.4",           // Virtual scrolling for large lists
  "react-window": "^1.8.10",            // Windowing for performance
  "react-beautiful-dnd": "^13.1.1",     // Drag and drop functionality
  "react-hotkeys-hook": "^4.5.0"        // Keyboard shortcuts
}
```

#### Form Handling & Validation
```json
{
  "react-hook-form": "^7.53.0",          // Form state management
  "zod": "^3.23.0",                     // Schema validation
  "@hookform/resolvers": "^3.4.0",      // Form validation resolvers
  "yup": "^1.4.0"                       // Alternative validation
}
```

#### HTTP Client & API Management
```json
{
  "axios": "^1.7.0",                    // HTTP client with interceptors
  "swr": "^2.2.5",                      // React data fetching (stale-while-revalidate)
  "react-query": "^5.59.0",             // TanStack Query for server state
  "@tanstack/react-query": "^5.59.0",   // Updated TanStack Query
  "openapi-typescript": "^7.4.0"        // TypeScript types from OpenAPI
}
```

#### Security & Authentication
```json
{
  "jsonwebtoken": "^9.0.2",             // JWT handling
  "jose": "^5.9.0",                     // Modern JWT library
  "crypto-js": "^4.2.0",                // Cryptographic functions
  "bcryptjs": "^2.4.3",                 // Password hashing
  "uuid": "^10.0.0"                     // UUID generation
}
```

#### Development & Testing
```json
{
  "@testing-library/user-event": "^14.5.0",
  "msw": "^2.4.0",                      // Mock Service Worker for API mocking
  "@faker-js/faker": "^9.0.0",          // Fake data generation
  "cypress": "^13.15.0",                // E2E testing
  "playwright": "^1.48.0",              // Alternative E2E testing
  "@playwright/test": "^1.48.0"
}
```

#### Performance & Monitoring
```json
{
  "web-vitals": "^5.1.0",               // Core Web Vitals tracking
  "@sentry/react": "^8.0.0",            // Error tracking and monitoring
  "@sentry/tracing": "^7.120.0",        // Performance monitoring
  "react-helmet-async": "^2.0.0",       // Document head management
  "workbox-webpack-plugin": "^7.1.0"    // Service worker generation
}
```

#### Utility Libraries
```json
{
  "lodash-es": "^4.17.21",              // Utility functions (ESM)
  "date-fns": "^4.1.0",                 // Date manipulation
  "numeral": "^2.0.6",                  // Number formatting
  "filesize": "^10.1.0",                // File size formatting
  "mime-types": "^2.1.35",              // MIME type detection
  "path-to-regexp": "^8.2.0",           // Path matching
  "query-string": "^9.1.0",             // URL query string parsing
  "url-join": "^5.0.0"                  // URL joining utility
}
```

## Component Architecture

### Shared UI Components

#### Design System Primitives
```typescript
// src/components/ui/primitives/
interface ButtonProps {
  variant: 'primary' | 'secondary' | 'danger' | 'ghost';
  size: 'sm' | 'md' | 'lg';
  disabled?: boolean;
  loading?: boolean;
  onClick: () => void;
  children: React.ReactNode;
}

interface InputProps {
  type: 'text' | 'email' | 'password' | 'number';
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  error?: string;
  disabled?: boolean;
  required?: boolean;
}
```

#### Layout Components
```typescript
// src/components/layout/
interface DashboardLayoutProps {
  children: React.ReactNode;
  sidebar?: React.ReactNode;
  header?: React.ReactNode;
  footer?: React.ReactNode;
}

interface GridProps {
  columns: number | { sm: number; md: number; lg: number };
  gap: string;
  children: React.ReactNode;
}
```

### Dashboard-Specific Components

#### Council Oversight Components
```typescript
// src/components/council/
interface VerdictCardProps {
  verdict: Verdict;
  onReview: (verdictId: string) => void;
  onOverride: (verdictId: string) => void;
}

interface DecisionFlowDiagramProps {
  verdictId: string;
  judges: Judge[];
  consensus: ConsensusResult;
  onNodeClick: (nodeId: string) => void;
}

interface EthicalAssessmentPanelProps {
  verdictId: string;
  assessments: EthicalAssessment[];
  onConcernClick: (concernId: string) => void;
}
```

#### Apple Silicon Components
```typescript
// src/components/apple-silicon/
interface HardwareMetricsPanelProps {
  metrics: HardwareMetrics;
  onThresholdChange: (metric: string, threshold: number) => void;
}

interface ThermalStatusDashboardProps {
  thermalData: ThermalMetrics;
  onCoolingAdjust: (settings: CoolingSettings) => void;
}

interface ModelRoutingVisualizerProps {
  routingDecisions: RoutingDecision[];
  hardwareStatus: HardwareStatus;
  onRoutingChange: (routing: RoutingConfig) => void;
}
```

#### Security Components
```typescript
// src/components/security/
interface AuthMonitoringDashboardProps {
  events: AuthEvent[];
  sessions: Session[];
  onTerminateSession: (sessionId: string) => void;
}

interface ThreatDetectionPanelProps {
  alerts: SecurityAlert[];
  threats: ThreatEvent[];
  onAlertAcknowledge: (alertId: string) => void;
}

interface AccessControlMatrixProps {
  roles: Role[];
  permissions: Permission[];
  onPermissionChange: (roleId: string, permissionId: string, granted: boolean) => void;
}
```

## API Specifications

### Core API Client Architecture

```typescript
// src/lib/api/client.ts
interface ApiClientConfig {
  baseUrl: string;
  timeout: number;
  retries: number;
  headers: Record<string, string>;
}

class ApiClient {
  constructor(config: ApiClientConfig);

  async get<T>(endpoint: string, params?: Record<string, any>): Promise<T>;
  async post<T>(endpoint: string, data: any): Promise<T>;
  async put<T>(endpoint: string, data: any): Promise<T>;
  async patch<T>(endpoint: string, data: any): Promise<T>;
  async delete(endpoint: string): Promise<void>;
}
```

### Service-Specific API Clients

```typescript
// src/lib/api/council.ts
interface CouncilApiClient {
  getVerdicts(filters: VerdictFilters): Promise<VerdictListResponse>;
  getVerdict(id: string): Promise<Verdict>;
  overrideVerdict(id: string, reason: string): Promise<Verdict>;
  getEthicalAssessments(verdictId: string): Promise<EthicalAssessment[]>;
  getJudgeMetrics(): Promise<JudgeMetrics[]>;
}

// src/lib/api/apple-silicon.ts
interface AppleSiliconApiClient {
  getMetrics(): Promise<HardwareMetrics>;
  getThermalStatus(): Promise<ThermalMetrics>;
  adjustCooling(settings: CoolingSettings): Promise<void>;
  getModelPerformance(): Promise<ModelMetrics[]>;
  updateRoutingConfig(config: RoutingConfig): Promise<void>;
}

// src/lib/api/security.ts
interface SecurityApiClient {
  getAuthEvents(filters: AuthEventFilters): Promise<AuthEvent[]>;
  terminateSession(sessionId: string): Promise<void>;
  getAccessMatrix(): Promise<AccessMatrix>;
  updatePermissions(updates: PermissionUpdate[]): Promise<void>;
  getThreatAlerts(): Promise<SecurityAlert[]>;
}
```

## State Management Architecture

### Global State Structure

```typescript
// src/stores/dashboard.ts
interface DashboardState {
  // UI State
  sidebar: {
    collapsed: boolean;
    activeSection: string;
  };
  theme: 'light' | 'dark' | 'auto';
  notifications: Notification[];

  // Data State
  council: CouncilState;
  appleSilicon: AppleSiliconState;
  security: SecurityState;
  workspace: WorkspaceState;
  systemHealth: SystemHealthState;

  // Connection State
  connection: {
    status: 'connected' | 'disconnected' | 'reconnecting';
    lastConnected: Date | null;
    reconnectAttempts: number;
  };
}

interface CouncilState {
  verdicts: Verdict[];
  judges: Judge[];
  ethicalAssessments: EthicalAssessment[];
  loading: boolean;
  error: string | null;
}

interface AppleSiliconState {
  hardware: HardwareMetrics;
  thermal: ThermalMetrics;
  routing: RoutingConfig;
  alerts: HardwareAlert[];
}

interface SecurityState {
  auth: AuthMetrics;
  access: AccessControl;
  secrets: SecretsInventory;
  threats: ThreatDetection;
}
```

### State Management Hooks

```typescript
// src/hooks/useCouncil.ts
function useCouncil() {
  const verdicts = useDashboardStore(state => state.council.verdicts);
  const judges = useDashboardStore(state => state.council.judges);
  const loading = useDashboardStore(state => state.council.loading);

  const fetchVerdicts = useCallback(async (filters: VerdictFilters) => {
    // Implementation
  }, []);

  const overrideVerdict = useCallback(async (id: string, reason: string) => {
    // Implementation
  }, []);

  return {
    verdicts,
    judges,
    loading,
    fetchVerdicts,
    overrideVerdict
  };
}
```

## Real-time Communication Architecture

### WebSocket Manager

```typescript
// src/lib/websocket/manager.ts
interface WebSocketManager {
  connect(): Promise<void>;
  disconnect(): Promise<void>;
  subscribe<T>(topic: string, callback: (data: T) => void): () => void;
  publish(topic: string, data: any): void;
  getConnectionStatus(): ConnectionStatus;
}

interface ConnectionStatus {
  connected: boolean;
  url: string;
  lastHeartbeat: Date;
  reconnectAttempts: number;
}
```

### SSE Manager

```typescript
// src/lib/sse/manager.ts
interface SSEManager {
  connect(endpoint: string): Promise<void>;
  disconnect(): void;
  onMessage<T>(callback: (data: T) => void): () => void;
  onError(callback: (error: Event) => void): () => void;
  getConnectionStatus(): SSEStatus;
}

interface SSEStatus {
  connected: boolean;
  endpoint: string;
  lastMessage: Date;
  retryCount: number;
}
```

## Data Flow Architecture

### Request/Response Flow

```typescript
// src/lib/data-flow/request.ts
interface RequestFlow<TData, TResult> {
  execute(data: TData): Promise<TResult>;
  withRetry(count: number): RequestFlow<TData, TResult>;
  withTimeout(ms: number): RequestFlow<TData, TResult>;
  withCache(ttl: number): RequestFlow<TData, TResult>;
  withFallback(fallback: TResult): RequestFlow<TData, TResult>;
}

// Usage example
const fetchVerdicts = createRequestFlow<VerdictFilters, Verdict[]>()
  .withRetry(3)
  .withTimeout(5000)
  .withCache(30000)
  .withFallback([]);
```

### Data Transformation Pipeline

```typescript
// src/lib/data-flow/transform.ts
interface DataTransformer<TInput, TOutput> {
  transform(input: TInput): TOutput;
  validate(input: TInput): ValidationResult;
  normalize(input: TInput): TInput;
}

interface ValidationResult {
  valid: boolean;
  errors: string[];
  warnings: string[];
}

// Council verdict transformer
const verdictTransformer: DataTransformer<RawVerdict, Verdict> = {
  transform(raw): Verdict {
    return {
      id: raw.id,
      status: mapVerdictStatus(raw.status),
      judges: raw.judges.map(mapJudge),
      consensus: calculateConsensus(raw.judge_votes),
      ethicalAssessment: assessEthics(raw.claims),
      createdAt: new Date(raw.created_at)
    };
  },

  validate(raw): ValidationResult {
    const errors: string[] = [];
    if (!raw.id) errors.push('Missing verdict ID');
    if (!raw.judges?.length) errors.push('No judges assigned');
    return { valid: errors.length === 0, errors, warnings: [] };
  }
};
```

## Testing Infrastructure

### Test Utilities

```typescript
// src/test/utils/
export function renderWithProviders(component: React.ReactNode) {
  // Implementation
}

export function createMockApiResponse<T>(data: T, status = 200) {
  // Implementation
}

export function mockWebSocket() {
  // Implementation
}

export function generateMockVerdict(): Verdict {
  // Implementation
}
```

### Test Configuration

```json
// jest.config.js
{
  "testEnvironment": "jsdom",
  "setupFilesAfterEnv": ["<rootDir>/jest.setup.js"],
  "moduleNameMapping": {
    "^@/(.*)$": "<rootDir>/src/$1"
  },
  "collectCoverageFrom": [
    "src/**/*.{ts,tsx}",
    "!src/**/*.d.ts"
  ],
  "coverageThreshold": {
    "global": {
      "branches": 80,
      "functions": 80,
      "lines": 80,
      "statements": 80
    }
  }
}
```

## Performance Optimizations

### Bundle Optimization

```typescript
// next.config.js
module.exports = {
  experimental: {
    optimizePackageImports: ['lucide-react', '@heroicons/react']
  },
  webpack: (config) => {
    // Custom webpack optimizations
    config.optimization.splitChunks = {
      chunks: 'all',
      cacheGroups: {
        vendor: {
          test: /[\\/]node_modules[\\/]/,
          name: 'vendors',
          chunks: 'all'
        },
        council: {
          test: /[\\/]components[\\/]council[\\/]/,
          name: 'council',
          chunks: 'all'
        }
      }
    };
    return config;
  }
};
```

### Caching Strategy

```typescript
// src/lib/cache/
interface CacheConfig {
  ttl: number;
  maxSize: number;
  strategy: 'lru' | 'lfu' | 'fifo';
}

class ApiCache {
  constructor(config: CacheConfig);

  get<T>(key: string): T | null;
  set<T>(key: string, value: T): void;
  invalidate(pattern: string): void;
  clear(): void;
}
```

## Security Implementation

### Authentication Flow

```typescript
// src/lib/auth/
interface AuthConfig {
  issuer: string;
  audience: string;
  clientId: string;
  scope: string[];
}

class AuthManager {
  constructor(config: AuthConfig);

  async login(credentials: LoginCredentials): Promise<AuthResult>;
  async logout(): Promise<void>;
  async refreshToken(): Promise<TokenResult>;
  isAuthenticated(): boolean;
  getUser(): User | null;
  getToken(): string | null;
}
```

### Authorization Guards

```typescript
// src/components/auth/
interface ProtectedRouteProps {
  children: React.ReactNode;
  roles?: string[];
  permissions?: string[];
  fallback?: React.ReactNode;
}

function ProtectedRoute({ children, roles, permissions, fallback }: ProtectedRouteProps) {
  const { user, isAuthenticated } = useAuth();

  if (!isAuthenticated) {
    return <Navigate to="/login" />;
  }

  if (roles && !hasRole(user, roles)) {
    return fallback || <AccessDenied />;
  }

  if (permissions && !hasPermission(user, permissions)) {
    return fallback || <AccessDenied />;
  }

  return <>{children}</>;
}
```

## Deployment Configuration

### Environment Variables

```bash
# .env.local
NEXT_PUBLIC_API_BASE_URL=http://localhost:8080/api
NEXT_PUBLIC_WS_URL=ws://localhost:8080/ws
NEXT_PUBLIC_SENTRY_DSN=your-sentry-dsn
NEXT_PUBLIC_GRAFANA_URL=http://localhost:3001
DATABASE_URL=postgresql://localhost:5432/dashboard
REDIS_URL=redis://localhost:6379
JWT_SECRET=your-jwt-secret
```

### Docker Configuration

```dockerfile
# Dockerfile
FROM node:20-alpine AS base

# Install dependencies
FROM base AS deps
RUN apk add --no-cache libc6-compat
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci --only=production

# Build application
FROM base AS builder
WORKDIR /app
COPY --from=deps /app/node_modules ./node_modules
COPY . .
RUN npm run build

# Production image
FROM base AS runner
WORKDIR /app

ENV NODE_ENV=production
ENV NEXT_TELEMETRY_DISABLED=1

RUN addgroup --system --gid 1001 nodejs
RUN adduser --system --uid 1001 nextjs

COPY --from=builder /app/public ./public
COPY --from=builder --chown=nextjs:nodejs /app/.next/standalone ./
COPY --from=builder --chown=nextjs:nodejs /app/.next/static ./.next/static

USER nextjs

EXPOSE 3000
ENV PORT=3000

CMD ["node", "server.js"]
```

### Kubernetes Configuration

```yaml
# k8s/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: dashboard
spec:
  replicas: 3
  selector:
    matchLabels:
      app: dashboard
  template:
    metadata:
      labels:
        app: dashboard
    spec:
      containers:
      - name: dashboard
        image: agent-agency/dashboard:latest
        ports:
        - containerPort: 3000
        env:
        - name: NEXT_PUBLIC_API_BASE_URL
          value: "http://api-service:8080/api"
        - name: NEXT_PUBLIC_WS_URL
          value: "ws://api-service:8080/ws"
        resources:
          requests:
            memory: "256Mi"
            cpu: "100m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /api/health
            port: 3000
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /api/health
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 5
```

## Remaining Ambiguities & Action Items

### Ambiguities to Resolve

1. **API Response Schemas**: Need detailed OpenAPI/Swagger specifications
2. **WebSocket Message Formats**: Define exact message structures for real-time updates
3. **Error Handling**: Standardize error response formats across all APIs
4. **Caching Strategy**: Define TTL and invalidation rules for different data types
5. **Offline Support**: Define which features work offline and sync strategies

### Action Items

1. **Create API Specification Document** with detailed request/response schemas
2. **Define WebSocket Protocol** with message types and formats
3. **Establish Error Handling Standards** across all components
4. **Design Caching Architecture** with appropriate TTL values
5. **Implement Offline Capabilities** for critical features
6. **Create Component Interface Definitions** with TypeScript interfaces
7. **Establish Testing Standards** and mock data structures
8. **Define Monitoring and Alerting Rules** for the dashboard itself

This BOM provides the complete technical foundation for implementing the Agent Agency V3 dashboard. All major components, dependencies, and architectural decisions are specified with concrete implementations.
