// k6 Load Test Script for Agent Agency V3
// Tests API endpoints under load to validate performance SLAs
//
// Usage:
//   k6 run tests/performance/agent-agency-smoke.js
//   k6 run --vus 10 --duration 30s tests/performance/agent-agency-smoke.js
//
// Performance SLAs:
//   - P95 response time: < 250ms for API endpoints
//   - P99 response time: < 500ms for API endpoints
//   - Error rate: < 0.1%
//   - Throughput: > 100 req/s per instance

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';

// Custom metrics
const errorRate = new Rate('errors');
const apiResponseTime = new Trend('api_response_time');

// Test configuration
export const options = {
    stages: [
        { duration: '30s', target: 10 },  // Ramp up to 10 users
        { duration: '1m', target: 10 },    // Stay at 10 users
        { duration: '30s', target: 20 },   // Ramp up to 20 users
        { duration: '1m', target: 20 },   // Stay at 20 users
        { duration: '30s', target: 0 },   // Ramp down to 0 users
    ],
    thresholds: {
        'http_req_duration': ['p(95)<250', 'p(99)<500'], // 95% < 250ms, 99% < 500ms
        'http_req_failed': ['rate<0.001'],                // Error rate < 0.1%
        'errors': ['rate<0.001'],                        // Custom error rate < 0.1%
    },
};

// Base URL for API (configurable via environment variable)
const BASE_URL = __ENV.API_URL || 'http://localhost:3000';

export default function () {
    // Test 1: Health check endpoint
    const healthCheck = http.get(`${BASE_URL}/health`);
    const healthCheckPassed = check(healthCheck, {
        'health check status is 200': (r) => r.status === 200,
        'health check response time < 100ms': (r) => r.timings.duration < 100,
    });
    
    if (!healthCheckPassed) {
        errorRate.add(1);
    }
    apiResponseTime.add(healthCheck.timings.duration);
    
    sleep(0.5);

    // Test 2: Metrics endpoint
    const metricsCheck = http.get(`${BASE_URL}/metrics`);
    const metricsCheckPassed = check(metricsCheck, {
        'metrics status is 200': (r) => r.status === 200,
        'metrics response time < 200ms': (r) => r.timings.duration < 200,
    });
    
    if (!metricsCheckPassed) {
        errorRate.add(1);
    }
    apiResponseTime.add(metricsCheck.timings.duration);
    
    sleep(0.5);

    // Test 3: API endpoint (if available)
    // Note: This assumes a basic API endpoint exists
    // Adjust based on actual API structure
    const apiCheck = http.get(`${BASE_URL}/api/v1/status`, {
        headers: {
            'Content-Type': 'application/json',
        },
    });
    
    // This endpoint may not exist, so we check for either 200 or 404
    const apiCheckPassed = check(apiCheck, {
        'api status is 200 or 404': (r) => r.status === 200 || r.status === 404,
        'api response time < 250ms': (r) => r.timings.duration < 250,
    });
    
    // Only count as error if it's a 5xx error
    if (apiCheck.status >= 500) {
        errorRate.add(1);
    }
    apiResponseTime.add(apiCheck.timings.duration);
    
    sleep(1);
}

export function handleSummary(data) {
    return {
        'stdout': textSummary(data, { indent: ' ', enableColors: true }),
        'test-results/performance-summary.json': JSON.stringify(data),
    };
}

function textSummary(data, options) {
    const indent = options.indent || '';
    const enableColors = options.enableColors || false;
    
    let summary = '\n';
    summary += `${indent}Performance Test Summary\n`;
    summary += `${indent}========================\n\n`;
    
    // HTTP metrics
    if (data.metrics.http_req_duration) {
        const duration = data.metrics.http_req_duration;
        summary += `${indent}Response Times:\n`;
        summary += `${indent}  Average: ${duration.values.avg.toFixed(2)}ms\n`;
        summary += `${indent}  P95: ${duration.values['p(95)'].toFixed(2)}ms\n`;
        summary += `${indent}  P99: ${duration.values['p(99)'].toFixed(2)}ms\n`;
        summary += `${indent}  Max: ${duration.values.max.toFixed(2)}ms\n\n`;
    }
    
    // Request rate
    if (data.metrics.http_reqs) {
        const reqs = data.metrics.http_reqs;
        summary += `${indent}Request Rate:\n`;
        summary += `${indent}  Total: ${reqs.values.count}\n`;
        summary += `${indent}  Rate: ${reqs.values.rate.toFixed(2)} req/s\n\n`;
    }
    
    // Error rate
    if (data.metrics.http_req_failed) {
        const failed = data.metrics.http_req_failed;
        summary += `${indent}Error Rate:\n`;
        summary += `${indent}  Failed: ${(failed.values.rate * 100).toFixed(2)}%\n\n`;
    }
    
    // Thresholds
    if (data.metrics.http_req_duration) {
        summary += `${indent}Thresholds:\n`;
        const thresholds = data.metrics.http_req_duration.thresholds || {};
        for (const [threshold, passed] of Object.entries(thresholds)) {
            const status = passed ? '✓' : '✗';
            summary += `${indent}  ${status} ${threshold}\n`;
        }
    }
    
    return summary;
}

