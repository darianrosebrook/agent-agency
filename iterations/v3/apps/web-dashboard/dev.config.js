// Development configuration for Agent Agency V3 Web Dashboard
// This file provides default environment variables for development

module.exports = {
  // Mock Data Configuration
  NEXT_PUBLIC_USE_MOCK_DATA: true,

  // API Configuration
  NEXT_PUBLIC_API_BASE_URL: 'http://localhost:8080/api',

  // WebSocket Configuration
  NEXT_PUBLIC_WS_URL: 'ws://localhost:8080/ws',

  // Development Settings
  NEXT_PUBLIC_DEBUG: true,
  NEXT_PUBLIC_PERFORMANCE_MONITORING: true,

  // Mock Data Settings
  NEXT_PUBLIC_MOCK_DELAY: 100,
  NEXT_PUBLIC_MOCK_ERROR_RATE: 2,

  // Dashboard Configuration
  NEXT_PUBLIC_REALTIME_UPDATES: true,
  NEXT_PUBLIC_UPDATE_INTERVAL: 10000,

  // Feature Flags
  NEXT_PUBLIC_APPLE_SILICON_MONITORING: true,
  NEXT_PUBLIC_THERMAL_MONITORING: true,
  NEXT_PUBLIC_COUNCIL_OVERSIGHT: true,
  NEXT_PUBLIC_SECURITY_MONITORING: true,
  NEXT_PUBLIC_OBSERVABILITY: true,
  NEXT_PUBLIC_WORKSPACE_MANAGEMENT: true,

  // Connection Settings
  NEXT_PUBLIC_CONNECTION_TIMEOUT: 10000,
  NEXT_PUBLIC_MAX_RETRIES: 3,
  NEXT_PUBLIC_RETRY_DELAY: 1000,

  // Port Configuration (for reference - configured in package.json)
  PORT: 3002,
};
