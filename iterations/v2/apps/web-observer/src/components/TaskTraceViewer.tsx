"use client";

import { ObserverApiClient } from "@/lib/api-client";
import { useEffect, useState } from "react";

interface TaskTraceViewerProps {
  apiClient: ObserverApiClient;
}

interface TaskTrace {
  taskId: string;
  status: "running" | "completed" | "failed" | "cancelled";
  createdAt: string;
  startedAt?: string;
  completedAt?: string;
  duration?: number;
  agentId?: string;
  agentName?: string;
  steps: TaskStep[];
  errors: TaskError[];
  metadata: Record<string, any>;
}

interface TaskStep {
  id: string;
  type: "intake" | "processing" | "execution" | "validation" | "cleanup";
  status: "pending" | "running" | "completed" | "failed";
  startedAt: string;
  completedAt?: string;
  duration?: number;
  agentId?: string;
  description: string;
  metadata?: Record<string, any>;
}

interface TaskError {
  id: string;
  stepId: string;
  timestamp: string;
  error: string;
  stack?: string;
  context?: Record<string, any>;
}

export default function TaskTraceViewer({ apiClient }: TaskTraceViewerProps) {
  const [selectedTaskId, setSelectedTaskId] = useState<string>("");
  const [taskTrace, setTaskTrace] = useState<TaskTrace | null>(null);
  const [recentTasks, setRecentTasks] = useState<TaskTrace[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [expandedSteps, setExpandedSteps] = useState<Set<string>>(new Set());

  useEffect(() => {
    loadRecentTasks();
  }, []);

  useEffect(() => {
    if (selectedTaskId) {
      loadTaskTrace(selectedTaskId);
    }
  }, [selectedTaskId]);

  const loadRecentTasks = async () => {
    try {
      // Mock data - would come from backend API
      const mockTasks: TaskTrace[] = [
        {
          taskId: "task-001",
          status: "completed",
          createdAt: new Date(Date.now() - 300000).toISOString(),
          startedAt: new Date(Date.now() - 295000).toISOString(),
          completedAt: new Date(Date.now() - 120000).toISOString(),
          duration: 175000,
          agentId: "runtime-docsmith",
          agentName: "Documentation Smith",
          steps: [
            {
              id: "step-001",
              type: "intake",
              status: "completed",
              startedAt: new Date(Date.now() - 295000).toISOString(),
              completedAt: new Date(Date.now() - 290000).toISOString(),
              duration: 5000,
              description: "Task intake and validation",
            },
            {
              id: "step-002",
              type: "processing",
              status: "completed",
              startedAt: new Date(Date.now() - 290000).toISOString(),
              completedAt: new Date(Date.now() - 180000).toISOString(),
              duration: 110000,
              agentId: "runtime-docsmith",
              description: "Documentation analysis and generation",
            },
            {
              id: "step-003",
              type: "validation",
              status: "completed",
              startedAt: new Date(Date.now() - 180000).toISOString(),
              completedAt: new Date(Date.now() - 120000).toISOString(),
              duration: 60000,
              description: "Output validation and formatting",
            },
          ],
          errors: [],
          metadata: {
            priority: 5,
            type: "documentation",
            inputSize: 2048,
            outputSize: 8192,
          },
        },
        {
          taskId: "task-002",
          status: "failed",
          createdAt: new Date(Date.now() - 240000).toISOString(),
          startedAt: new Date(Date.now() - 235000).toISOString(),
          completedAt: new Date(Date.now() - 180000).toISOString(),
          duration: 55000,
          agentId: "runtime-refactorer",
          agentName: "Refactor Sage",
          steps: [
            {
              id: "step-004",
              type: "intake",
              status: "completed",
              startedAt: new Date(Date.now() - 235000).toISOString(),
              completedAt: new Date(Date.now() - 230000).toISOString(),
              duration: 5000,
              description: "Task intake and validation",
            },
            {
              id: "step-005",
              type: "processing",
              status: "failed",
              startedAt: new Date(Date.now() - 230000).toISOString(),
              completedAt: new Date(Date.now() - 180000).toISOString(),
              duration: 50000,
              agentId: "runtime-refactorer",
              description: "Code refactoring and optimization",
            },
          ],
          errors: [
            {
              id: "error-001",
              stepId: "step-005",
              timestamp: new Date(Date.now() - 180000).toISOString(),
              error: "SyntaxError: Unexpected token in refactored code",
              stack: "at RefactorEngine.process (/app/src/engine.js:123:45)",
              context: {
                file: "src/components/Button.tsx",
                line: 42,
                code: "const handleClick = useCallback(() => {",
              },
            },
          ],
          metadata: {
            priority: 3,
            type: "refactoring",
            filesAffected: 5,
            errorCount: 1,
          },
        },
      ];

      setRecentTasks(mockTasks);
    } catch (err) {
      console.error("Failed to load recent tasks:", err);
    }
  };

  const loadTaskTrace = async (taskId: string) => {
    setLoading(true);
    try {
      // Find the task in recent tasks or fetch from API
      const task = recentTasks.find(t => t.taskId === taskId);
      if (task) {
        setTaskTrace(task);
      } else {
        // Would fetch from backend API
        setError("Task trace not found");
      }
    } catch (err) {
      console.error("Failed to load task trace:", err);
      setError("Failed to load task trace");
    } finally {
      setLoading(false);
    }
  };

  const toggleStepExpansion = (stepId: string) => {
    const newExpanded = new Set(expandedSteps);
    if (newExpanded.has(stepId)) {
      newExpanded.delete(stepId);
    } else {
      newExpanded.add(stepId);
    }
    setExpandedSteps(newExpanded);
  };

  const getStepIcon = (type: string) => {
    switch (type) {
      case "intake":
        return "📥";
      case "processing":
        return "⚙️";
      case "execution":
        return "▶️";
      case "validation":
        return "✅";
      case "cleanup":
        return "🧹";
      default:
        return "📋";
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case "completed":
        return "text-green-600 bg-green-100";
      case "running":
        return "text-blue-600 bg-blue-100";
      case "failed":
        return "text-red-600 bg-red-100";
      case "pending":
        return "text-gray-600 bg-gray-100";
      case "cancelled":
        return "text-orange-600 bg-orange-100";
      default:
        return "text-gray-600 bg-gray-100";
    }
  };

  const formatDuration = (ms: number) => {
    if (ms < 1000) return `${ms}ms`;
    const seconds = Math.floor(ms / 1000);
    if (seconds < 60) return `${seconds}s`;
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = seconds % 60;
    return `${minutes}m ${remainingSeconds}s`;
  };

  return (
    <div className="bg-white rounded-lg shadow p-6">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-lg font-semibold text-gray-900">
          Task Trace Viewer
        </h2>
        <div className="flex items-center space-x-2">
          <span className="text-sm text-gray-500">
            {recentTasks.length} recent tasks
          </span>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Task List */}
        <div className="lg:col-span-1">
          <h3 className="text-md font-medium text-gray-900 mb-4">
            Recent Tasks
          </h3>
          <div className="space-y-3 max-h-96 overflow-y-auto">
            {recentTasks.map((task) => (
              <div
                key={task.taskId}
                onClick={() => setSelectedTaskId(task.taskId)}
                className={`p-3 border rounded-lg cursor-pointer transition-colors ${
                  selectedTaskId === task.taskId
                    ? "border-blue-500 bg-blue-50"
                    : "border-gray-200 hover:border-gray-300"
                }`}
              >
                <div className="flex items-center justify-between mb-2">
                  <span className="text-sm font-medium text-gray-900">
                    {task.taskId}
                  </span>
                  <span
                    className={`px-2 py-1 rounded text-xs font-medium ${getStatusColor(
                      task.status
                    )}`}
                  >
                    {task.status}
                  </span>
                </div>
                <div className="text-xs text-gray-600 space-y-1">
                  <div>
                    Created: {new Date(task.createdAt).toLocaleTimeString()}
                  </div>
                  {task.agentName && (
                    <div>Agent: {task.agentName}</div>
                  )}
                  {task.duration && (
                    <div>Duration: {formatDuration(task.duration)}</div>
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Task Trace Details */}
        <div className="lg:col-span-2">
          {loading ? (
            <div className="flex items-center justify-center h-64">
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
            </div>
          ) : error ? (
            <div className="text-center py-8">
              <div className="text-red-500 text-4xl mb-4">⚠️</div>
              <h3 className="text-lg font-medium text-gray-900 mb-2">
                Error Loading Trace
              </h3>
              <p className="text-gray-600">{error}</p>
            </div>
          ) : !taskTrace ? (
            <div className="text-center py-8">
              <div className="text-gray-400 text-4xl mb-4">🔍</div>
              <h3 className="text-lg font-medium text-gray-900 mb-2">
                Select a Task
              </h3>
              <p className="text-gray-600">
                Choose a task from the list to view its execution trace
              </p>
            </div>
          ) : (
            <div className="space-y-6">
              {/* Task Header */}
              <div className="border-b pb-4">
                <div className="flex items-center justify-between mb-4">
                  <h3 className="text-lg font-medium text-gray-900">
                    Task: {taskTrace.taskId}
                  </h3>
                  <span
                    className={`px-3 py-1 rounded-full text-sm font-medium ${getStatusColor(
                      taskTrace.status
                    )}`}
                  >
                    {taskTrace.status.toUpperCase()}
                  </span>
                </div>

                <div className="grid grid-cols-2 gap-4 text-sm">
                  <div>
                    <span className="text-gray-600">Created:</span>{" "}
                    {new Date(taskTrace.createdAt).toLocaleString()}
                  </div>
                  {taskTrace.startedAt && (
                    <div>
                      <span className="text-gray-600">Started:</span>{" "}
                      {new Date(taskTrace.startedAt).toLocaleString()}
                    </div>
                  )}
                  {taskTrace.agentName && (
                    <div>
                      <span className="text-gray-600">Agent:</span>{" "}
                      {taskTrace.agentName}
                    </div>
                  )}
                  {taskTrace.duration && (
                    <div>
                      <span className="text-gray-600">Duration:</span>{" "}
                      {formatDuration(taskTrace.duration)}
                    </div>
                  )}
                </div>
              </div>

              {/* Execution Timeline */}
              <div>
                <h4 className="text-md font-medium text-gray-900 mb-4">
                  Execution Timeline
                </h4>
                <div className="space-y-4">
                  {taskTrace.steps.map((step, index) => (
                    <div
                      key={step.id}
                      className="relative"
                    >
                      {/* Timeline line */}
                      {index < taskTrace.steps.length - 1 && (
                        <div className="absolute left-6 top-12 w-0.5 h-8 bg-gray-300"></div>
                      )}

                      <div className="flex items-start space-x-4">
                        {/* Step icon */}
                        <div
                          className={`w-12 h-12 rounded-full flex items-center justify-center text-lg ${
                            step.status === "completed"
                              ? "bg-green-100 text-green-600"
                              : step.status === "failed"
                              ? "bg-red-100 text-red-600"
                              : step.status === "running"
                              ? "bg-blue-100 text-blue-600"
                              : "bg-gray-100 text-gray-600"
                          }`}
                        >
                          {getStepIcon(step.type)}
                        </div>

                        {/* Step details */}
                        <div className="flex-1 min-w-0">
                          <div className="flex items-center justify-between mb-2">
                            <h5 className="text-sm font-medium text-gray-900">
                              {step.description}
                            </h5>
                            <div className="flex items-center space-x-2">
                              <span
                                className={`px-2 py-1 rounded text-xs font-medium ${getStatusColor(
                                  step.status
                                )}`}
                              >
                                {step.status}
                              </span>
                              {step.duration && (
                                <span className="text-xs text-gray-500">
                                  {formatDuration(step.duration)}
                                </span>
                              )}
                            </div>
                          </div>

                          <div className="text-xs text-gray-600 space-y-1">
                            <div>
                              Started: {new Date(step.startedAt).toLocaleString()}
                            </div>
                            {step.completedAt && (
                              <div>
                                Completed:{" "}
                                {new Date(step.completedAt).toLocaleString()}
                              </div>
                            )}
                            {step.agentId && (
                              <div>Agent ID: {step.agentId}</div>
                            )}
                          </div>

                          {/* Expandable metadata */}
                          {step.metadata && Object.keys(step.metadata).length > 0 && (
                            <div className="mt-2">
                              <button
                                onClick={() => toggleStepExpansion(step.id)}
                                className="text-xs text-blue-600 hover:text-blue-800"
                              >
                                {expandedSteps.has(step.id) ? "Hide" : "Show"} Details
                              </button>
                              {expandedSteps.has(step.id) && (
                                <div className="mt-2 p-2 bg-gray-50 rounded text-xs">
                                  <pre className="whitespace-pre-wrap">
                                    {JSON.stringify(step.metadata, null, 2)}
                                  </pre>
                                </div>
                              )}
                            </div>
                          )}
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              </div>

              {/* Errors */}
              {taskTrace.errors.length > 0 && (
                <div>
                  <h4 className="text-md font-medium text-gray-900 mb-4">
                    Errors ({taskTrace.errors.length})
                  </h4>
                  <div className="space-y-3">
                    {taskTrace.errors.map((error) => (
                      <div
                        key={error.id}
                        className="p-4 bg-red-50 border border-red-200 rounded"
                      >
                        <div className="flex items-start space-x-3">
                          <span className="text-red-600 text-lg">❌</span>
                          <div className="flex-1">
                            <div className="text-sm font-medium text-red-800 mb-1">
                              {error.error}
                            </div>
                            <div className="text-xs text-red-600 mb-2">
                              {new Date(error.timestamp).toLocaleString()}
                            </div>
                            {error.stack && (
                              <details className="text-xs">
                                <summary className="cursor-pointer text-red-700 hover:text-red-900">
                                  Stack Trace
                                </summary>
                                <pre className="mt-2 p-2 bg-red-100 rounded text-red-800 whitespace-pre-wrap">
                                  {error.stack}
                                </pre>
                              </details>
                            )}
                          </div>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {/* Task Metadata */}
              <div>
                <h4 className="text-md font-medium text-gray-900 mb-4">
                  Task Metadata
                </h4>
                <div className="bg-gray-50 p-4 rounded">
                  <pre className="text-xs text-gray-800 whitespace-pre-wrap">
                    {JSON.stringify(taskTrace.metadata, null, 2)}
                  </pre>
                </div>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
