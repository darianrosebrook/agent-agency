#!/usr/bin/env bash
# NextJS Interface Update Script for V3
# Updates the web dashboard to work with V3 backend services
# @darianrosebrook

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
V3_ROOT="$PROJECT_ROOT/iterations/v3"
DASHBOARD_DIR="$V3_ROOT/apps/web-dashboard"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

log_step() {
    echo -e "${BLUE}🔄 $1${NC}"
}

# Check if dashboard directory exists
check_dashboard() {
    if [[ ! -d "$DASHBOARD_DIR" ]]; then
        log_error "Dashboard directory not found: $DASHBOARD_DIR"
        exit 1
    fi
    log_success "Dashboard directory found"
}

# Update package.json with V3-specific scripts
update_package_json() {
    log_step "Updating package.json for V3 integration..."
    
    cd "$DASHBOARD_DIR"
    
    # Create backup
    cp package.json package.json.backup
    
    # Add V3-specific scripts
    cat > package.json.v3 << 'EOF'
{
  "name": "agent-agency-v3-dashboard",
  "version": "0.1.0",
  "private": true,
  "description": "Agent Agency V3 Web Dashboard for monitoring and managing agent task execution",
  "keywords": [
    "agent",
    "agency",
    "dashboard",
    "nextjs",
    "react",
    "typescript",
    "v3"
  ],
  "author": "Agent Agency Team",
  "license": "MIT",
  "scripts": {
    "dev": "next dev",
    "dev:v3": "next dev -p 3000",
    "build": "next build",
    "build:v3": "next build && next export",
    "start": "next start",
    "start:v3": "next start -p 3000",
    "lint": "next lint",
    "lint:fix": "next lint --fix",
    "type-check": "tsc --noEmit",
    "format": "prettier --write .",
    "format:check": "prettier --check .",
    "test": "jest",
    "test:watch": "jest --watch",
    "test:coverage": "jest --coverage",
    "clean": "rm -rf .next out dist",
    "analyze": "cross-env ANALYZE=true next build",
    "export": "next export",
    "serve": "npx serve out",
    "v3:setup": "npm install && npm run type-check",
    "v3:dev": "npm run dev:v3",
    "v3:build": "npm run build:v3",
    "v3:start": "npm run start:v3"
  },
  "dependencies": {
    "@react-three/drei": "^10.7.6",
    "@react-three/fiber": "^9.4.0",
    "@types/d3": "^7.4.3",
    "@types/node": "^22.0.0",
    "@types/react": "^19.0.0",
    "@types/react-dom": "^19.0.0",
    "@types/three": "^0.180.0",
    "autoprefixer": "^10.4.0",
    "class-variance-authority": "^0.7.0",
    "clsx": "^2.1.1",
    "critters": "^0.0.23",
    "d3": "^7.9.0",
    "gsap": "^3.13.0",
    "lucide-react": "^0.548.0",
    "next": "^16.0.0",
    "postcss": "^8.4.0",
    "react": "^19.0.0",
    "react-dom": "^19.0.0",
    "sass": "^1.69.0",
    "tailwind-merge": "^3.3.1",
    "three": "^0.180.0",
    "typescript": "^5.0.0",
    "web-vitals": "^5.1.0",
    "zustand": "^5.0.8"
  },
  "devDependencies": {
    "@eslint/eslintrc": "^3.3.1",
    "@eslint/js": "^9.38.0",
    "@next/bundle-analyzer": "^16.0.0",
    "@testing-library/jest-dom": "^6.0.0",
    "@testing-library/react": "^16.0.0",
    "@typescript-eslint/eslint-plugin": "^8.46.2",
    "@typescript-eslint/parser": "^8.46.2",
    "cross-env": "^10.0.0",
    "eslint": "^9.0.0",
    "eslint-config-next": "^16.0.0",
    "eslint-plugin-jsx-a11y": "^6.10.2",
    "eslint-plugin-react": "^7.37.5",
    "eslint-plugin-react-hooks": "^7.0.1",
    "jest": "^30.0.0",
    "jest-environment-jsdom": "^30.0.0",
    "prettier": "^3.0.0",
    "serve": "^14.0.0"
  },
  "type": "module",
  "engines": {
    "node": ">=20.0.0",
    "npm": ">=10.0.0"
  },
  "browserslist": {
    "production": [
      ">0.2%",
      "not dead",
      "not op_mini all"
    ],
    "development": [
      "last 1 chrome version",
      "last 1 firefox version",
      "last 1 safari version"
    ]
  }
}
EOF
    
    # Replace package.json
    mv package.json.v3 package.json
    log_success "package.json updated with V3 scripts"
}

# Create V3 environment configuration
create_env_config() {
    log_step "Creating V3 environment configuration..."
    
    cd "$DASHBOARD_DIR"
    
    cat > .env.local << 'EOF'
# Agent Agency V3 Dashboard Configuration
NEXT_PUBLIC_API_BASE_URL=http://localhost:8080
NEXT_PUBLIC_API_VERSION=v1
NEXT_PUBLIC_WS_URL=ws://localhost:8080
NEXT_PUBLIC_ENVIRONMENT=development

# V3 Backend Services
NEXT_PUBLIC_API_SERVER_URL=http://localhost:8080
NEXT_PUBLIC_WORKER_SYSTEM_URL=http://localhost:8081
NEXT_PUBLIC_DATABASE_URL=postgresql://postgres:agent_agency_secure_password_123@localhost:5432/agent_agency
NEXT_PUBLIC_REDIS_URL=redis://localhost:6379

# Feature Flags
NEXT_PUBLIC_ENABLE_CHAT=true
NEXT_PUBLIC_ENABLE_METRICS=true
NEXT_PUBLIC_ENABLE_ALERTS=true
NEXT_PUBLIC_ENABLE_TASK_MANAGEMENT=true

# Development Settings
NEXT_PUBLIC_DEBUG_MODE=true
NEXT_PUBLIC_LOG_LEVEL=info
EOF
    
    log_success "Environment configuration created"
}

# Update Next.js configuration for V3
update_next_config() {
    log_step "Updating Next.js configuration..."
    
    cd "$DASHBOARD_DIR"
    
    # Create backup
    cp next.config.js next.config.js.backup
    
    cat > next.config.js << 'EOF'
/** @type {import('next').NextConfig} */
const nextConfig = {
  // V3-specific configuration
  reactStrictMode: true,
  swcMinify: true,
  
  // API proxy configuration
  async rewrites() {
    return [
      {
        source: '/api/v1/:path*',
        destination: 'http://localhost:8080/api/v1/:path*',
      },
      {
        source: '/ws/:path*',
        destination: 'ws://localhost:8080/ws/:path*',
      },
    ];
  },
  
  // Environment variables
  env: {
    CUSTOM_KEY: process.env.CUSTOM_KEY,
  },
  
  // Performance optimizations
  experimental: {
    optimizeCss: true,
    optimizePackageImports: ['lucide-react', 'd3'],
  },
  
  // Build configuration
  output: 'standalone',
  
  // Image optimization
  images: {
    domains: ['localhost'],
    unoptimized: process.env.NODE_ENV === 'development',
  },
  
  // Webpack configuration
  webpack: (config, { dev, isServer }) => {
    // V3-specific webpack optimizations
    if (!dev && !isServer) {
      config.optimization.splitChunks.cacheGroups = {
        ...config.optimization.splitChunks.cacheGroups,
        vendor: {
          test: /[\\/]node_modules[\\/]/,
          name: 'vendors',
          chunks: 'all',
        },
      };
    }
    
    return config;
  },
};

module.exports = nextConfig;
EOF
    
    log_success "Next.js configuration updated"
}

# Create V3-specific API client
create_api_client() {
    log_step "Creating V3 API client..."
    
    cd "$DASHBOARD_DIR/src/lib"
    
    cat > v3-api-client.ts << 'EOF'
/**
 * V3 API Client for Agent Agency Dashboard
 * Handles communication with V3 backend services
 */

export interface V3ApiConfig {
  baseUrl: string;
  apiVersion: string;
  timeout?: number;
}

export interface TaskSubmissionRequest {
  description: string;
  context?: string;
  priority?: string;
}

export interface TaskSubmissionResponse {
  task_id: string;
  status: string;
  message: string;
}

export interface Task {
  id: string;
  title: string;
  description: string;
  status: string;
  priority: string;
  createdAt: string;
  updatedAt: string;
  acceptanceCriteria: string[];
  events: any[];
}

export interface HealthStatus {
  status: string;
  service: string;
  version: string;
  timestamp: string;
  components: Record<string, string>;
}

export interface Metrics {
  metrics: {
    active_tasks: number;
    completed_tasks: number;
    failed_tasks: number;
    avg_response_time_ms: number;
  };
  status: string;
}

export class V3ApiClient {
  private config: V3ApiConfig;

  constructor(config: V3ApiConfig) {
    this.config = config;
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const url = `${this.config.baseUrl}/api/${this.config.apiVersion}${endpoint}`;
    
    const response = await fetch(url, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...options.headers,
      },
      signal: AbortSignal.timeout(this.config.timeout || 10000),
    });

    if (!response.ok) {
      throw new Error(`API request failed: ${response.status} ${response.statusText}`);
    }

    return response.json();
  }

  // Health check
  async getHealth(): Promise<HealthStatus> {
    return this.request<HealthStatus>('/health');
  }

  // Task management
  async submitTask(task: TaskSubmissionRequest): Promise<TaskSubmissionResponse> {
    return this.request<TaskSubmissionResponse>('/tasks', {
      method: 'POST',
      body: JSON.stringify(task),
    });
  }

  async getTasks(): Promise<{ tasks: Task[]; total: number }> {
    return this.request<{ tasks: Task[]; total: number }>('/tasks');
  }

  async getTask(taskId: string): Promise<Task> {
    return this.request<Task>(`/tasks/${taskId}`);
  }

  async pauseTask(taskId: string): Promise<void> {
    await this.request(`/tasks/${taskId}/pause`, { method: 'POST' });
  }

  async resumeTask(taskId: string): Promise<void> {
    await this.request(`/tasks/${taskId}/resume`, { method: 'POST' });
  }

  async cancelTask(taskId: string): Promise<void> {
    await this.request(`/tasks/${taskId}/cancel`, { method: 'POST' });
  }

  // Metrics
  async getMetrics(): Promise<Metrics> {
    return this.request<Metrics>('/metrics');
  }

  // Chat
  async createChatSession(userId?: string): Promise<{ sessionId: string; sessionUuid: string; createdAt: string }> {
    return this.request('/chat/session', {
      method: 'POST',
      body: JSON.stringify({ userId }),
    });
  }

  async getChatMessages(sessionId: string, limit?: number): Promise<{ messages: any[]; total_count: number }> {
    const params = limit ? `?limit=${limit}` : '';
    return this.request(`/chat/messages/${sessionId}${params}`);
  }

  async sendChatMessage(sessionId: string, message: string, sender?: string): Promise<any> {
    return this.request(`/chat/message/${sessionId}`, {
      method: 'POST',
      body: JSON.stringify({ message, sender }),
    });
  }

  // Alerts
  async getActiveAlerts(): Promise<{ alerts: any[]; total: number }> {
    return this.request('/alerts');
  }

  async acknowledgeAlert(alertId: string): Promise<void> {
    await this.request(`/alerts/${alertId}/acknowledge`, { method: 'POST' });
  }

  async resolveAlert(alertId: string): Promise<void> {
    await this.request(`/alerts/${alertId}/resolve`, { method: 'POST' });
  }
}

// Default client instance
export const v3ApiClient = new V3ApiClient({
  baseUrl: process.env.NEXT_PUBLIC_API_BASE_URL || 'http://localhost:8080',
  apiVersion: process.env.NEXT_PUBLIC_API_VERSION || 'v1',
  timeout: 10000,
});

// WebSocket client for real-time updates
export class V3WebSocketClient {
  private ws: WebSocket | null = null;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private reconnectDelay = 1000;

  constructor(private sessionId: string) {}

  connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      const wsUrl = `${process.env.NEXT_PUBLIC_WS_URL || 'ws://localhost:8080'}/api/v1/chat/ws/${this.sessionId}`;
      
      this.ws = new WebSocket(wsUrl);
      
      this.ws.onopen = () => {
        this.reconnectAttempts = 0;
        resolve();
      };
      
      this.ws.onerror = (error) => {
        reject(error);
      };
      
      this.ws.onclose = () => {
        this.handleReconnect();
      };
    });
  }

  private handleReconnect() {
    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      this.reconnectAttempts++;
      setTimeout(() => {
        this.connect().catch(console.error);
      }, this.reconnectDelay * this.reconnectAttempts);
    }
  }

  sendMessage(message: string, sender?: string) {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify({ message, sender }));
    }
  }

  onMessage(callback: (data: any) => void) {
    if (this.ws) {
      this.ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          callback(data);
        } catch (error) {
          console.error('Failed to parse WebSocket message:', error);
        }
      };
    }
  }

  disconnect() {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }
}
EOF
    
    log_success "V3 API client created"
}

# Install dependencies
install_dependencies() {
    log_step "Installing dependencies..."
    
    cd "$DASHBOARD_DIR"
    
    if [[ ! -d "node_modules" ]]; then
        npm install
        log_success "Dependencies installed"
    else
        log_info "Dependencies already installed"
    fi
}

# Run type check
run_type_check() {
    log_step "Running TypeScript type check..."
    
    cd "$DASHBOARD_DIR"
    
    if npm run type-check; then
        log_success "Type check passed"
    else
        log_warning "Type check failed - some types may need updating"
    fi
}

# Main update function
update_dashboard() {
    log_info "Updating NextJS Dashboard for V3 integration..."
    
    check_dashboard
    update_package_json
    create_env_config
    update_next_config
    create_api_client
    install_dependencies
    run_type_check
    
    log_success "Dashboard updated for V3 integration!"
    log_info "You can now run:"
    log_info "  cd $DASHBOARD_DIR"
    log_info "  npm run v3:dev"
    log_info ""
    log_info "Or use the unified startup script:"
    log_info "  ../../scripts/v3/start-v3-system.sh start"
}

# Main script logic
case "${1:-update}" in
    "update")
        update_dashboard
        ;;
    "dev")
        cd "$DASHBOARD_DIR"
        npm run v3:dev
        ;;
    "build")
        cd "$DASHBOARD_DIR"
        npm run v3:build
        ;;
    "start")
        cd "$DASHBOARD_DIR"
        npm run v3:start
        ;;
    *)
        echo "Usage: $0 {update|dev|build|start}"
        echo ""
        echo "Commands:"
        echo "  update - Update dashboard for V3 integration"
        echo "  dev    - Start development server"
        echo "  build  - Build for production"
        echo "  start  - Start production server"
        exit 1
        ;;
esac
