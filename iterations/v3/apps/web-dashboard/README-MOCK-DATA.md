# Mock Data Development Setup

This document explains how to use mock data for development when the backend API is not available.

## Overview

The web dashboard includes comprehensive mock data for all API endpoints, allowing full development and testing of the dashboard without a running backend.

## Setup

### 1. Environment Configuration

Create a `.env.local` file in the `apps/web-dashboard` directory:

```bash
# Copy the example file
cp env.example .env.local
```

### 2. Enable Mock Data

In your `.env.local` file, set:

```env
NEXT_PUBLIC_USE_MOCK_DATA=true
```

### 3. Development Mode

Ensure you're running in development mode:

```bash
npm run dev
```

## Mock Data Files

The following mock data files are available in `src/mock-data/`:

- **`council-api-mock.json`** - Council verdicts, judges, ethical assessments
- **`apple-silicon-api-mock.json`** - Hardware metrics, thermal data, routing decisions
- **`observability-api-mock.json`** - System metrics, alerts, logs, traces
- **`workspace-api-mock.json`** - Git status, file changes, workspace health
- **`security-api-mock.json`** - Authentication, policies, security events
- **`vector-database-api-mock.json`** - Vector embeddings, search results, clusters
- **`task-api-mock.json`** - Task management, assignments, progress tracking
- **`database-api-mock.json`** - Database performance, tables, queries, backups
- **`chat-api-mock.json`** - Conversations, messages, participants, templates
- **`tts-api-mock.json`** - Text-to-speech voices, synthesis jobs, audio files
- **`analytics-api-mock.json`** - User analytics, page metrics, feature usage
- **`metrics-api-mock.json`** - Agent performance, system metrics, health checks

## Features

### Realistic Data
- All mock data is designed to be realistic and representative
- Includes edge cases and various data states
- Covers success, warning, and error scenarios

### Development Simulation
- **Network Delays**: Configurable delays to simulate real API calls
- **Error Simulation**: Occasional errors to test error handling
- **Real-time Updates**: WebSocket simulation for live data

### Environment Variables

```env
# Mock Data Configuration
NEXT_PUBLIC_USE_MOCK_DATA=true

# Simulation Settings
NEXT_PUBLIC_MOCK_DELAY=100          # Network delay in milliseconds
NEXT_PUBLIC_MOCK_ERROR_RATE=5       # Error rate percentage (0-100)

# Feature Flags
NEXT_PUBLIC_APPLE_SILICON_MONITORING=true
NEXT_PUBLIC_COUNCIL_OVERSIGHT=true
NEXT_PUBLIC_SECURITY_MONITORING=true
NEXT_PUBLIC_OBSERVABILITY=true
NEXT_PUBLIC_WORKSPACE_MANAGEMENT=true
```

## API Integration

The mock data loader automatically integrates with existing API clients:

```typescript
// Council API automatically uses mock data when enabled
const verdicts = await councilApiClient.getVerdicts();

// Apple Silicon API with mock data
const metrics = await appleSiliconApiClient.getCurrentMetrics();

// All APIs support mock data
const stats = await observabilityApiClient.getSystemMetrics();
```

## Development Workflow

### 1. Start Development Server

```bash
cd apps/web-dashboard
npm run dev
```

### 2. Access Dashboard

Navigate to `http://localhost:3000` to see the dashboard with mock data.

### 3. Test Features

- **Council Dashboard**: View verdicts, judge metrics, ethical assessments
- **Apple Silicon Dashboard**: Monitor hardware utilization, thermal management
- **Observability Dashboard**: System metrics, alerts, performance data
- **Workspace Dashboard**: Git status, file changes, development metrics
- **Security Dashboard**: Authentication, policies, audit logs
- **Vector Database Dashboard**: Embeddings, search results, clusters, analytics
- **Task Management Dashboard**: Task tracking, assignments, progress monitoring
- **Database Dashboard**: Performance metrics, table statistics, query analysis
- **Chat Dashboard**: Conversations, messages, participants, templates
- **TTS Dashboard**: Voices, synthesis jobs, audio files, usage statistics
- **Analytics Dashboard**: User metrics, page analytics, feature usage
- **Metrics Dashboard**: Agent performance, system metrics, health monitoring

## Mock Data Customization

### Adding New Data

1. Edit the relevant mock JSON file in `src/mock-data/`
2. Update the corresponding mock API in `src/lib/mock-data-loader.ts`
3. Restart the development server

### Modifying Behavior

Edit `src/lib/mock-data-loader.ts` to:
- Change network delays
- Modify error rates
- Add new simulation features

## Production Safety

Mock data is **automatically disabled** in production:

- Only works when `NODE_ENV=development`
- Requires explicit `NEXT_PUBLIC_USE_MOCK_DATA=true`
- Fails gracefully if mock data is unavailable

## Troubleshooting

### Mock Data Not Loading

1. Check environment variables:
   ```bash
   echo $NODE_ENV
   echo $NEXT_PUBLIC_USE_MOCK_DATA
   ```

2. Verify file paths:
   ```bash
   ls -la src/mock-data/
   ```

3. Check browser console for errors

### API Errors

If you see API errors with mock data enabled:

1. Check the mock data files are valid JSON
2. Verify the mock API methods match the real API signatures
3. Look for TypeScript errors in the mock data loader

### Performance Issues

If the dashboard is slow with mock data:

1. Reduce `NEXT_PUBLIC_MOCK_DELAY`
2. Set `NEXT_PUBLIC_MOCK_ERROR_RATE=0`
3. Check for infinite loops in component re-renders

## Data Structure

Each mock data file follows the same structure as the real API responses:

```typescript
// Example: council-api-mock.json
{
  "verdicts": [...],           // Array of verdict objects
  "judges": [...],            // Array of judge objects
  "ethicalAssessments": [...], // Array of assessment objects
  "councilStats": {...}       // Statistics object
}
```

## Next Steps

Once the backend API is available:

1. Set `NEXT_PUBLIC_USE_MOCK_DATA=false`
2. Configure `NEXT_PUBLIC_API_BASE_URL` to point to your backend
3. Update WebSocket URL in `NEXT_PUBLIC_WS_URL`
4. Test with real data

The dashboard will automatically switch from mock data to real API calls without any code changes.
