#!/usr/bin/env node

/**
 * Test script to verify dashboard connectivity to V3 backend
 * Tests both REST API and real-time metrics streaming
 */

const https = require('https');
const http = require('http');

const V3_HOST = process.env.V3_BACKEND_HOST || 'localhost';
const V3_PORT = process.env.V3_BACKEND_PORT || 8080;
const V3_BASE_URL = `http://${V3_HOST}:${V3_PORT}`;

console.log('Testing Dashboard ↔ V3 Backend Connectivity');
console.log('================================================');
console.log(`V3 Backend URL: ${V3_BASE_URL}`);
console.log();

async function makeRequest(url, options = {}) {
  return new Promise((resolve, reject) => {
    const protocol = url.startsWith('https:') ? https : http;

    const request = protocol.get(url, options, (res) => {
      let data = '';

      res.on('data', (chunk) => {
        data += chunk;
      });

      res.on('end', () => {
        try {
          const jsonData = JSON.parse(data);
          resolve({ status: res.statusCode, data: jsonData, headers: res.headers });
        } catch (e) {
          resolve({ status: res.statusCode, data, headers: res.headers });
        }
      });
    });

    request.on('error', (err) => {
      reject(err);
    });

    // Timeout after 10 seconds
    request.setTimeout(10000, () => {
      request.destroy();
      reject(new Error('Request timeout'));
    });
  });
}

async function testHealthEndpoint() {
  console.log('1️⃣ Testing Health Endpoint...');
  try {
    const response = await makeRequest(`${V3_BASE_URL}/health`);
    if (response.status === 200) {
      console.log('Health endpoint responding');
      return true;
    } else {
      console.log(`Health endpoint returned status ${response.status}`);
      return false;
    }
  } catch (error) {
    console.log(`Health endpoint error: ${error.message}`);
    return false;
  }
}

async function testMetricsEndpoint() {
  console.log('2️⃣ Testing Metrics Endpoint...');
  try {
    const response = await makeRequest(`${V3_BASE_URL}/api/v1/metrics`);
    if (response.status === 200) {
      console.log('Metrics endpoint responding');
      if (response.data && response.data.metrics) {
        console.log(`   Found ${Object.keys(response.data.metrics).length} metric types`);
      }
      return true;
    } else {
      console.log(`Metrics endpoint returned status ${response.status}`);
      return false;
    }
  } catch (error) {
    console.log(`Metrics endpoint error: ${error.message}`);
    return false;
  }
}

async function testMetricsStream() {
  console.log('3️⃣ Testing Real-time Metrics Stream...');
  return new Promise((resolve) => {
    let receivedData = false;
    let timeout;

    const protocol = V3_BASE_URL.startsWith('https:') ? https : http;
    const streamUrl = `${V3_BASE_URL}/api/v1/metrics/stream`;

    const request = protocol.get(streamUrl, {
      headers: {
        'Accept': 'text/event-stream',
        'Cache-Control': 'no-cache',
      }
    }, (res) => {
      if (res.statusCode !== 200) {
        console.log(`Metrics stream returned status ${res.statusCode}`);
        resolve(false);
        return;
      }

      console.log('Metrics stream connected');

      let buffer = '';

      res.on('data', (chunk) => {
        buffer += chunk.toString();

        // Check for complete SSE messages
        const lines = buffer.split('\n');
        for (const line of lines) {
          if (line.startsWith('data: ')) {
            try {
              const data = JSON.parse(line.substring(6));
              if (data.timestamp && data.metrics) {
                console.log('Received real-time metrics data');
                console.log(`   CPU: ${data.metrics.cpu_usage_percent?.toFixed(1) || 'N/A'}%`);
                console.log(`   Memory: ${data.metrics.memory_usage_percent?.toFixed(1) || 'N/A'}%`);
                console.log(`   ⚙️ Active Tasks: ${data.metrics.active_tasks || 0}`);
                receivedData = true;
                clearTimeout(timeout);
                resolve(true);
                return;
              }
            } catch (e) {
              // Not a complete JSON message yet
            }
          }
        }
      });

      res.on('end', () => {
        if (!receivedData) {
          console.log('Metrics stream ended without receiving data');
          resolve(false);
        }
      });
    });

    request.on('error', (err) => {
      console.log(`Metrics stream connection error: ${err.message}`);
      resolve(false);
    });

    // Set timeout for receiving data
    timeout = setTimeout(() => {
      if (!receivedData) {
        console.log('Timeout waiting for metrics stream data');
        request.destroy();
        resolve(false);
      }
    }, 15000); // 15 second timeout
  });
}

async function testApiTasksEndpoint() {
  console.log('4️⃣ Testing Tasks API Endpoint...');
  try {
    const response = await makeRequest(`${V3_BASE_URL}/api/v1/tasks`);
    if (response.status === 200) {
      console.log('Tasks API endpoint responding');
      if (response.data && Array.isArray(response.data.tasks)) {
        console.log(`   Found ${response.data.tasks.length} tasks`);
      }
      return true;
    } else {
      console.log(`Tasks API endpoint returned status ${response.status}`);
      return false;
    }
  } catch (error) {
    console.log(`Tasks API endpoint error: ${error.message}`);
    return false;
  }
}

async function runTests() {
  const results = [];

  results.push(await testHealthEndpoint());
  results.push(await testMetricsEndpoint());
  results.push(await testMetricsStream());
  results.push(await testApiTasksEndpoint());

  console.log();
  console.log('================================================');
  console.log('Test Results Summary');
  console.log('================================================');

  const passed = results.filter(r => r).length;
  const total = results.length;

  console.log(`Tests Passed: ${passed}/${total}`);

  if (passed === total) {
    console.log('All tests passed! Dashboard should work correctly.');
    console.log();
    console.log('Next steps:');
    console.log('1. Start the dashboard: cd apps/web-dashboard && npm run dev');
    console.log('2. Open http://localhost:3000 in your browser');
    console.log('3. Check the connection status indicator in the header');
    console.log('4. Navigate to the Metrics page to see real-time data');
    process.exit(0);
  } else {
    console.log('Some tests failed. Dashboard may not work correctly.');
    console.log();
    console.log('Troubleshooting:');
    console.log('1. Ensure V3 backend is running on the correct host/port');
    console.log('2. Check V3_BACKEND_HOST environment variable');
    console.log('3. Verify V3 backend logs for any errors');
    console.log('4. Test V3 endpoints manually with curl');
    process.exit(1);
  }
}

// Handle script interruption
process.on('SIGINT', () => {
  console.log('\n⏹️ Test interrupted by user');
  process.exit(130);
});

process.on('SIGTERM', () => {
  console.log('\n⏹️ Test terminated');
  process.exit(143);
});

// Run the tests
runTests().catch((error) => {
  console.error('Test script failed:', error);
  process.exit(1);
});
