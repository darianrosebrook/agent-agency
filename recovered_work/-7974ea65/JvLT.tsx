"use client";

import { ObserverApiClient } from "@/lib/api-client";
import { useState } from "react";

interface HealthCheckRunnerProps {
  apiClient: ObserverApiClient;
}

interface HealthCheck {
  id: string;
  name: string;
  description: string;
  category: "system" | "database" | "network" | "agents" | "performance";
  status: "pending" | "running" | "passed" | "failed" | "warning";
  duration?: number;
  message?: string;
  details?: Record<string, any>;
  recommendations?: string[];
}

interface HealthCheckResult {
  overallStatus: "healthy" | "degraded" | "critical";
  totalChecks: number;
  passedChecks: number;
  failedChecks: number;
  warningChecks: number;
  checks: HealthCheck[];
  timestamp: string;
  duration: number;
}

export default function HealthCheckRunner({ apiClient }: HealthCheckRunnerProps) {
  const [isRunning, setIsRunning] = useState(false);
  const [result, setResult] = useState<HealthCheckResult | null>(null);
  const [progress, setProgress] = useState<HealthCheck[]>([]);
  const [selectedCategory, setSelectedCategory] = useState<string>("all");

  const runHealthChecks = async () => {
    setIsRunning(true);
    setResult(null);
    setProgress([]);

    const startTime = Date.now();
    const allChecks: HealthCheck[] = [
      // System checks
      {
        id: "cpu-usage",
        name: "CPU Usage Check",
        description: "Verify CPU usage is within acceptable limits",
        category: "system",
        status: "pending",
      },
      {
        id: "memory-usage",
        name: "Memory Usage Check",
        description: "Verify memory usage is within acceptable limits",
        category: "system",
        status: "pending",
      },
      {
        id: "disk-space",
        name: "Disk Space Check",
        description: "Verify sufficient disk space is available",
        category: "system",
        status: "pending",
      },

      // Database checks
      {
        id: "db-connection",
        name: "Database Connection",
        description: "Verify database connectivity",
        category: "database",
        status: "pending",
      },
      {
        id: "db-performance",
        name: "Database Performance",
        description: "Check database query performance",
        category: "database",
        status: "pending",
      },
      {
        id: "db-integrity",
        name: "Database Integrity",
        description: "Verify database schema and data integrity",
        category: "database",
        status: "pending",
      },

      // Network checks
      {
        id: "network-connectivity",
        name: "Network Connectivity",
        description: "Verify network connectivity and latency",
        category: "network",
        status: "pending",
      },
      {
        id: "api-endpoints",
        name: "API Endpoints",
        description: "Test API endpoint availability and response times",
        category: "network",
        status: "pending",
      },

      // Agent checks
      {
        id: "agent-health",
        name: "Agent Health",
        description: "Verify all agents are healthy and responsive",
        category: "agents",
        status: "pending",
      },
      {
        id: "agent-load",
        name: "Agent Load Balance",
        description: "Check agent load distribution",
        category: "agents",
        status: "pending",
      },

      // Performance checks
      {
        id: "response-times",
        name: "Response Times",
        description: "Verify API response times are within SLA",
        category: "performance",
        status: "pending",
      },
      {
        id: "throughput",
        name: "System Throughput",
        description: "Check system throughput and capacity",
        category: "performance",
        status: "pending",
      },
      {
        id: "error-rates",
        name: "Error Rates",
        description: "Verify error rates are within acceptable limits",
        category: "performance",
        status: "pending",
      },
    ];

    // Run checks sequentially with progress updates
    for (const check of allChecks) {
      await runSingleCheck(check);
    }

    const endTime = Date.now();
    const finalResult: HealthCheckResult = {
      overallStatus: calculateOverallStatus(progress),
      totalChecks: progress.length,
      passedChecks: progress.filter(c => c.status === "passed").length,
      failedChecks: progress.filter(c => c.status === "failed").length,
      warningChecks: progress.filter(c => c.status === "warning").length,
      checks: progress,
      timestamp: new Date().toISOString(),
      duration: endTime - startTime,
    };

    setResult(finalResult);
    setIsRunning(false);
  };

  const runSingleCheck = async (check: HealthCheck) => {
    // Update status to running
    check.status = "running";
    setProgress(prev => [...prev.filter(c => c.id !== check.id), check]);

    // Simulate check execution with different results based on check type
    await new Promise(resolve => setTimeout(resolve, 500 + Math.random() * 1500));

    // Mock results - in real implementation, these would call actual health check functions
    switch (check.id) {
      case "cpu-usage":
        check.status = Math.random() > 0.8 ? "warning" : "passed";
        check.duration = 200;
        check.message = check.status === "warning" ? "CPU usage at 78%" : "CPU usage normal";
        if (check.status === "warning") {
          check.recommendations = ["Monitor CPU usage trends", "Consider scaling if usage continues to rise"];
        }
        break;

      case "memory-usage":
        check.status = Math.random() > 0.9 ? "failed" : "passed";
        check.duration = 150;
        check.message = check.status === "failed" ? "Memory usage at 92%" : "Memory usage normal";
        if (check.status === "failed") {
          check.recommendations = ["Increase memory allocation", "Optimize memory usage", "Restart services if needed"];
        }
        break;

      case "disk-space":
        check.status = Math.random() > 0.95 ? "warning" : "passed";
        check.duration = 100;
        check.message = check.status === "warning" ? "Disk usage at 85%" : "Disk space sufficient";
        break;

      case "db-connection":
        check.status = Math.random() > 0.95 ? "failed" : "passed";
        check.duration = 300;
        check.message = check.status === "failed" ? "Database connection failed" : "Database connection successful";
        if (check.status === "failed") {
          check.recommendations = ["Check database server status", "Verify connection credentials", "Check network connectivity"];
        }
        break;

      case "db-performance":
        check.status = Math.random() > 0.85 ? "warning" : "passed";
        check.duration = 500;
        check.message = check.status === "warning" ? "Slow query detected" : "Database performance good";
        break;

      case "network-connectivity":
        check.status = "passed";
        check.duration = 200;
        check.message = "Network connectivity good";
        break;

      case "api-endpoints":
        check.status = Math.random() > 0.9 ? "warning" : "passed";
        check.duration = 400;
        check.message = check.status === "warning" ? "Some API endpoints slow" : "All API endpoints responding";
        break;

      case "agent-health":
        check.status = Math.random() > 0.9 ? "failed" : "passed";
        check.duration = 600;
        check.message = check.status === "failed" ? "One agent unresponsive" : "All agents healthy";
        if (check.status === "failed") {
          check.details = { unhealthyAgent: "runtime-refactorer" };
          check.recommendations = ["Restart unresponsive agent", "Check agent logs", "Verify agent dependencies"];
        }
        break;

      case "response-times":
        check.status = Math.random() > 0.8 ? "warning" : "passed";
        check.duration = 300;
        check.message = check.status === "warning" ? "P95 response time: 450ms" : "Response times within SLA";
        break;

      case "throughput":
        check.status = "passed";
        check.duration = 250;
        check.message = "System throughput normal";
        break;

      case "error-rates":
        check.status = Math.random() > 0.85 ? "failed" : "passed";
        check.duration = 200;
        check.message = check.status === "failed" ? "Error rate: 12%" : "Error rate within limits";
        break;

      default:
        check.status = "passed";
        check.duration = 100;
        check.message = "Check completed successfully";
    }

    setProgress(prev => [...prev.filter(c => c.id !== check.id), check]);
  };

  const calculateOverallStatus = (checks: HealthCheck[]): "healthy" | "degraded" | "critical" => {
    const failed = checks.filter(c => c.status === "failed").length;
    const warnings = checks.filter(c => c.status === "warning").length;

    if (failed > 0) return "critical";
    if (warnings > 2) return "degraded";
    return "healthy";
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case "passed":
        return "text-green-600 bg-green-100";
      case "warning":
        return "text-yellow-600 bg-yellow-100";
      case "failed":
        return "text-red-600 bg-red-100";
      case "running":
        return "text-blue-600 bg-blue-100";
      default:
        return "text-gray-600 bg-gray-100";
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case "passed":
        return "✅";
      case "warning":
        return "⚠️";
      case "failed":
        return "❌";
      case "running":
        return "🔄";
      default:
        return "⏳";
    }
  };

  const getCategoryIcon = (category: string) => {
    switch (category) {
      case "system":
        return "🖥️";
      case "database":
        return "🗄️";
      case "network":
        return "🌐";
      case "agents":
        return "🤖";
      case "performance":
        return "📊";
      default:
        return "🔧";
    }
  };

  const filteredChecks = result?.checks.filter(check =>
    selectedCategory === "all" || check.category === selectedCategory
  ) || [];

  const categories = ["all", "system", "database", "network", "agents", "performance"];

  return (
    <div className="bg-white rounded-lg shadow p-6">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h2 className="text-lg font-semibold text-gray-900">
            Health Check Runner
          </h2>
          <p className="text-sm text-gray-600 mt-1">
            Automated system diagnostics and health verification
          </p>
        </div>
        <div className="flex items-center space-x-4">
          {result && (
            <div className="text-right">
              <div className="text-sm text-gray-500">
                Completed in {(result.duration / 1000).toFixed(1)}s
              </div>
              <div
                className={`text-sm font-medium ${
                  result.overallStatus === "healthy"
                    ? "text-green-600"
                    : result.overallStatus === "degraded"
                    ? "text-yellow-600"
                    : "text-red-600"
                }`}
              >
                {result.overallStatus.toUpperCase()}
              </div>
            </div>
          )}
          <button
            onClick={runHealthChecks}
            disabled={isRunning}
            className={`px-4 py-2 text-sm font-medium rounded-md ${
              isRunning
                ? "bg-gray-100 text-gray-500 cursor-not-allowed"
                : "bg-blue-600 text-white hover:bg-blue-700"
            } transition-colors`}
          >
            {isRunning ? (
              <span className="flex items-center">
                <svg
                  className="animate-spin -ml-1 mr-2 h-4 w-4 text-gray-500"
                  fill="none"
                  viewBox="0 0 24 24"
                >
                  <circle
                    className="opacity-25"
                    cx="12"
                    cy="12"
                    r="10"
                    stroke="currentColor"
                    strokeWidth="4"
                  />
                  <path
                    className="opacity-75"
                    fill="currentColor"
                    d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                  />
                </svg>
                Running Checks...
              </span>
            ) : (
              "Run Health Checks"
            )}
          </button>
        </div>
      </div>

      {/* Results Summary */}
      {result && (
        <div className="mb-6 p-4 bg-gray-50 rounded-lg">
          <div className="grid grid-cols-4 gap-4 text-center">
            <div>
              <div className="text-2xl font-bold text-green-600">
                {result.passedChecks}
              </div>
              <div className="text-sm text-gray-600">Passed</div>
            </div>
            <div>
              <div className="text-2xl font-bold text-yellow-600">
                {result.warningChecks}
              </div>
              <div className="text-sm text-gray-600">Warnings</div>
            </div>
            <div>
              <div className="text-2xl font-bold text-red-600">
                {result.failedChecks}
              </div>
              <div className="text-sm text-gray-600">Failed</div>
            </div>
            <div>
              <div className="text-2xl font-bold text-gray-900">
                {result.totalChecks}
              </div>
              <div className="text-sm text-gray-600">Total</div>
            </div>
          </div>
        </div>
      )}

      {/* Category Filter */}
      <div className="mb-6">
        <div className="flex items-center space-x-2">
          <label className="text-sm font-medium text-gray-700">
            Filter by Category:
          </label>
          <select
            value={selectedCategory}
            onChange={(e) => setSelectedCategory(e.target.value)}
            className="text-sm border border-gray-300 rounded px-2 py-1"
          >
            {categories.map(category => (
              <option key={category} value={category}>
                {category.charAt(0).toUpperCase() + category.slice(1)}
              </option>
            ))}
          </select>
        </div>
      </div>

      {/* Progress/Results */}
      <div className="space-y-3">
        {(progress.length > 0 ? progress : result?.checks || []).map((check) => (
          <div
            key={check.id}
            className={`p-4 border rounded-lg ${
              check.status === "failed"
                ? "border-red-200 bg-red-50"
                : check.status === "warning"
                ? "border-yellow-200 bg-yellow-50"
                : check.status === "passed"
                ? "border-green-200 bg-green-50"
                : "border-gray-200 bg-gray-50"
            }`}
          >
            <div className="flex items-center justify-between mb-2">
              <div className="flex items-center space-x-3">
                <span className="text-lg">
                  {getCategoryIcon(check.category)}
                </span>
                <div>
                  <h3 className="text-sm font-medium text-gray-900">
                    {check.name}
                  </h3>
                  <p className="text-xs text-gray-600">{check.description}</p>
                </div>
              </div>

              <div className="flex items-center space-x-2">
                {check.duration && (
                  <span className="text-xs text-gray-500">
                    {check.duration}ms
                  </span>
                )}
                <span
                  className={`px-2 py-1 rounded text-xs font-medium ${getStatusColor(
                    check.status
                  )}`}
                >
                  {getStatusIcon(check.status)} {check.status}
                </span>
              </div>
            </div>

            {check.message && (
              <div className="text-sm text-gray-700 mb-2">{check.message}</div>
            )}

            {check.recommendations && check.recommendations.length > 0 && (
              <div className="mt-2">
                <div className="text-xs font-medium text-gray-700 mb-1">
                  Recommendations:
                </div>
                <ul className="text-xs text-gray-600 space-y-1">
                  {check.recommendations.map((rec, i) => (
                    <li key={i} className="flex items-start">
                      <span className="text-blue-600 mr-1">•</span>
                      {rec}
                    </li>
                  ))}
                </ul>
              </div>
            )}

            {check.details && Object.keys(check.details).length > 0 && (
              <div className="mt-2">
                <details className="text-xs">
                  <summary className="cursor-pointer text-gray-700 hover:text-gray-900">
                    Show Details
                  </summary>
                  <pre className="mt-2 p-2 bg-white rounded text-gray-800 whitespace-pre-wrap">
                    {JSON.stringify(check.details, null, 2)}
                  </pre>
                </details>
              </div>
            )}
          </div>
        ))}
      </div>

      {!isRunning && progress.length === 0 && (
        <div className="text-center py-12">
          <div className="text-gray-400 text-4xl mb-4">🔍</div>
          <h3 className="text-lg font-medium text-gray-900 mb-2">
            Health Check Ready
          </h3>
          <p className="text-gray-600 mb-4">
            Click "Run Health Checks" to perform comprehensive system diagnostics
          </p>
          <button
            onClick={runHealthChecks}
            className="px-6 py-3 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors"
          >
            Start Health Check
          </button>
        </div>
      )}
    </div>
  );
}
