// K6 Load Testing Script for Multimodal RAG System
// Tests the complete multimodal RAG workflow under load

import http from "k6/http";
import { check, sleep } from "k6";
import { Rate, Trend, Counter } from "k6/metrics";

// Custom metrics
const multimodalProcessingRate = new Rate("multimodal_processing_success_rate");
const vectorSearchTrend = new Trend("vector_search_duration");
const embeddingGenerationTrend = new Trend("embedding_generation_duration");
const crossModalValidationTrend = new Trend("cross_modal_validation_duration");

// Test configuration
export const options = {
  stages: [
    { duration: "2m", target: 10 }, // Ramp up to 10 users
    { duration: "5m", target: 10 }, // Stay at 10 users
    { duration: "2m", target: 50 }, // Ramp up to 50 users
    { duration: "5m", target: 50 }, // Stay at 50 users
    { duration: "2m", target: 100 }, // Ramp up to 100 users
    { duration: "5m", target: 100 }, // Stay at 100 users
    { duration: "2m", target: 0 }, // Ramp down to 0 users
  ],
  thresholds: {
    http_req_duration: ["p(95)<2000"], // 95% of requests must complete below 2s
    http_req_failed: ["rate<0.05"], // Error rate must be below 5%
    multimodal_processing_success_rate: ["rate>0.95"], // 95% success rate
    vector_search_duration: ["p(95)<1000"], // 95% of vector searches < 1s
    embedding_generation_duration: ["p(95)<500"], // 95% of embeddings < 500ms
  },
};

// Test data
const testDocuments = [
  {
    content:
      "The quick brown fox jumps over the lazy dog. This is a test document for multimodal RAG processing.",
    modality: "text",
    type: "document",
  },
  {
    content:
      "Machine learning is a subset of artificial intelligence that focuses on algorithms that can learn from data.",
    modality: "text",
    type: "document",
  },
  {
    content:
      "Computer vision enables machines to interpret and understand visual information from the world.",
    modality: "text",
    type: "document",
  },
];

const testQueries = [
  "What is machine learning?",
  "How does computer vision work?",
  "Explain artificial intelligence",
  "What are neural networks?",
  "How do transformers work?",
  "What is natural language processing?",
  "Explain deep learning",
  "What is reinforcement learning?",
];

// Helper function to get random element from array
function getRandomElement(array) {
  return array[Math.floor(Math.random() * array.length)];
}

// Helper function to generate test file content
function generateTestFile(modality) {
  switch (modality) {
    case "text":
      return {
        content: getRandomElement(testDocuments).content,
        filename: `test_document_${Date.now()}.txt`,
        content_type: "text/plain",
      };
    case "image":
      return {
        content: "base64_encoded_image_data_placeholder",
        filename: `test_image_${Date.now()}.jpg`,
        content_type: "image/jpeg",
      };
    case "audio":
      return {
        content: "base64_encoded_audio_data_placeholder",
        filename: `test_audio_${Date.now()}.mp3`,
        content_type: "audio/mpeg",
      };
    default:
      return {
        content: getRandomElement(testDocuments).content,
        filename: `test_file_${Date.now()}.txt`,
        content_type: "text/plain",
      };
  }
}

export default function () {
  const baseUrl = __ENV.BASE_URL || "http://localhost:8080";
  const apiKey = __ENV.API_KEY || "test-api-key";

  const headers = {
    "Content-Type": "application/json",
    Authorization: `Bearer ${apiKey}`,
    "X-API-Key": apiKey,
  };

  // Test 1: Health Check
  const healthResponse = http.get(`${baseUrl}/health`);
  check(healthResponse, {
    "health check status is 200": (r) => r.status === 200,
    "health check response time < 100ms": (r) => r.timings.duration < 100,
  });

  // Test 2: Vector Search
  const query = getRandomElement(testQueries);
  const searchStartTime = Date.now();

  const searchResponse = http.post(
    `${baseUrl}/api/v1/search`,
    JSON.stringify({
      query: query,
      modality: "text",
      limit: 10,
      threshold: 0.7,
    }),
    { headers }
  );

  const searchDuration = Date.now() - searchStartTime;
  vectorSearchTrend.add(searchDuration);

  check(searchResponse, {
    "vector search status is 200": (r) => r.status === 200,
    "vector search returns results": (r) => {
      const body = JSON.parse(r.body);
      return body.results && Array.isArray(body.results);
    },
    "vector search response time < 1s": (r) => r.timings.duration < 1000,
  });

  // Test 3: Multimodal Processing
  const modalities = ["text", "image", "audio"];
  const modality = getRandomElement(modalities);
  const testFile = generateTestFile(modality);

  const processingStartTime = Date.now();

  const processingResponse = http.post(
    `${baseUrl}/api/v1/process`,
    JSON.stringify({
      content: testFile.content,
      filename: testFile.filename,
      content_type: testFile.content_type,
      modality: modality,
      project_scope: "load-test",
    }),
    { headers }
  );

  const processingDuration = Date.now() - processingStartTime;

  const processingSuccess = check(processingResponse, {
    "multimodal processing status is 200": (r) => r.status === 200,
    "multimodal processing returns job_id": (r) => {
      const body = JSON.parse(r.body);
      return body.job_id && typeof body.job_id === "string";
    },
    "multimodal processing response time < 2s": (r) =>
      r.timings.duration < 2000,
  });

  multimodalProcessingRate.add(processingSuccess);

  // Test 4: Embedding Generation
  const embeddingStartTime = Date.now();

  const embeddingResponse = http.post(
    `${baseUrl}/api/v1/embeddings`,
    JSON.stringify({
      text: query,
      model: "e5-small-v2",
    }),
    { headers }
  );

  const embeddingDuration = Date.now() - embeddingStartTime;
  embeddingGenerationTrend.add(embeddingDuration);

  check(embeddingResponse, {
    "embedding generation status is 200": (r) => r.status === 200,
    "embedding generation returns vector": (r) => {
      const body = JSON.parse(r.body);
      return body.embedding && Array.isArray(body.embedding);
    },
    "embedding generation response time < 500ms": (r) =>
      r.timings.duration < 500,
  });

  // Test 5: Cross-modal Validation
  const validationStartTime = Date.now();

  const validationResponse = http.post(
    `${baseUrl}/api/v1/validate`,
    JSON.stringify({
      claim: "This document discusses machine learning concepts",
      evidence: [
        { content: query, modality: "text", source: "search_result" },
        {
          content: testFile.content,
          modality: modality,
          source: "uploaded_file",
        },
      ],
      validation_type: "cross_modal",
    }),
    { headers }
  );

  const validationDuration = Date.now() - validationStartTime;
  crossModalValidationTrend.add(validationDuration);

  check(validationResponse, {
    "cross-modal validation status is 200": (r) => r.status === 200,
    "cross-modal validation returns score": (r) => {
      const body = JSON.parse(r.body);
      return body.consistency_score !== undefined;
    },
    "cross-modal validation response time < 1s": (r) =>
      r.timings.duration < 1000,
  });

  // Test 6: Batch Processing
  if (Math.random() < 0.3) {
    // 30% chance of batch processing
    const batchResponse = http.post(
      `${baseUrl}/api/v1/batch/process`,
      JSON.stringify({
        files: [
          generateTestFile("text"),
          generateTestFile("text"),
          generateTestFile("text"),
        ],
        project_scope: "load-test-batch",
      }),
      { headers }
    );

    check(batchResponse, {
      "batch processing status is 200": (r) => r.status === 200,
      "batch processing returns batch_id": (r) => {
        const body = JSON.parse(r.body);
        return body.batch_id && typeof body.batch_id === "string";
      },
    });
  }

  // Test 7: Metrics Endpoint
  const metricsResponse = http.get(`${baseUrl}/metrics`);
  check(metricsResponse, {
    "metrics endpoint status is 200": (r) => r.status === 200,
    "metrics endpoint returns prometheus format": (r) =>
      r.body.includes("# HELP"),
  });

  // Random sleep between requests
  sleep(Math.random() * 2 + 0.5); // 0.5-2.5 seconds
}

export function handleSummary(data) {
  return {
    "load-test-results.json": JSON.stringify(data, null, 2),
    "load-test-summary.html": generateHtmlReport(data),
  };
}

function generateHtmlReport(data) {
  return `
    <!DOCTYPE html>
    <html>
    <head>
        <title>Multimodal RAG Load Test Results</title>
        <style>
            body { font-family: Arial, sans-serif; margin: 20px; }
            .metric { margin: 10px 0; padding: 10px; border: 1px solid #ddd; }
            .success { background-color: #d4edda; }
            .warning { background-color: #fff3cd; }
            .error { background-color: #f8d7da; }
        </style>
    </head>
    <body>
        <h1>Multimodal RAG Load Test Results</h1>
        <div class="metric">
            <h3>Test Summary</h3>
            <p>Duration: ${data.state.testRunDurationMs / 1000}s</p>
            <p>VUs: ${data.metrics.vus.values.max}</p>
            <p>Iterations: ${data.metrics.iterations.values.count}</p>
        </div>
        <div class="metric">
            <h3>HTTP Metrics</h3>
            <p>Total Requests: ${data.metrics.http_reqs.values.count}</p>
            <p>Failed Requests: ${data.metrics.http_req_failed.values.count}</p>
            <p>Average Response Time: ${data.metrics.http_req_duration.values.avg.toFixed(
              2
            )}ms</p>
            <p>95th Percentile: ${data.metrics.http_req_duration.values[
              "p(95)"
            ].toFixed(2)}ms</p>
        </div>
        <div class="metric">
            <h3>Custom Metrics</h3>
            <p>Multimodal Processing Success Rate: ${(
              data.metrics.multimodal_processing_success_rate.values.rate * 100
            ).toFixed(2)}%</p>
            <p>Average Vector Search Time: ${data.metrics.vector_search_duration.values.avg.toFixed(
              2
            )}ms</p>
            <p>Average Embedding Generation Time: ${data.metrics.embedding_generation_duration.values.avg.toFixed(
              2
            )}ms</p>
        </div>
    </body>
    </html>
  `;
}
