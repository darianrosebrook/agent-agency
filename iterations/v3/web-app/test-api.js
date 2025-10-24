#!/usr/bin/env node

/**
 * Simple API connectivity test for Agent Agency V3 Web Interface
 * Tests all endpoints that the web interface depends on
 */

const API_BASE = "http://localhost:8080/api";

async function testEndpoint(name, url, options = {}) {
  console.log(`🔍 Testing ${name}...`);

  try {
    const response = await fetch(url, options);
    const success = response.ok;

    console.log(
      `   ${success ? "✅" : "❌"} ${response.status} ${response.statusText}`
    );

    if (success && !options.method) {
      // Try to parse JSON for GET requests
      try {
        const data = await response.json();
        console.log(
          `   📊 Response: ${JSON.stringify(data, null, 2).slice(0, 200)}...`
        );
      } catch (e) {
        console.log(
          `   📄 Response: ${await response.text().slice(0, 200)}...`
        );
      }
    }

    return success;
  } catch (error) {
    console.log(`   ❌ Error: ${error.message}`);
    return false;
  }
}

async function runTests() {
  console.log("🧪 Agent Agency V3 API Connectivity Test");
  console.log("==========================================\n");

  const tests = [
    {
      name: "System Health",
      url: `${API_BASE}/health`,
    },
    {
      name: "Active Tasks",
      url: `${API_BASE}/tasks/active`,
    },
    {
      name: "Task Status (sample)",
      url: `${API_BASE}/tasks/550e8400-e29b-41d4-a716-446655440000/status`,
    },
  ];

  let passed = 0;
  let total = tests.length;

  for (const test of tests) {
    if (await testEndpoint(test.name, test.url, test.options)) {
      passed++;
    }
    console.log();
  }

  console.log(`📊 Results: ${passed}/${total} tests passed`);

  if (passed === total) {
    console.log("🎉 All API endpoints are accessible!");
    console.log(
      "\n🌐 You can now open the web interface at: http://localhost:3000"
    );
  } else {
    console.log("⚠️  Some API endpoints are not accessible.");
    console.log("   Make sure the API server is running on port 8080");
  }

  return passed === total;
}

// Run tests if this script is executed directly
if (require.main === module) {
  runTests()
    .then((success) => {
      process.exit(success ? 0 : 1);
    })
    .catch((error) => {
      console.error("❌ Test runner failed:", error);
      process.exit(1);
    });
}

module.exports = { runTests, testEndpoint };
