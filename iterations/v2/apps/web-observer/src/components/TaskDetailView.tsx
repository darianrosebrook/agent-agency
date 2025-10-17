import type { ChainOfThoughtEntry, Task } from "@/types/api";
import { useState } from "react";
import ObservationForm from "./ObservationForm";

interface TaskDetailViewProps {
  task: Task;
  chainOfThought: ChainOfThoughtEntry[];
  onClose: () => void;
  onAddObservation: (message: string, author?: string) => Promise<void>;
}

export default function TaskDetailView({
  task,
  chainOfThought,
  onClose,
  onAddObservation,
}: TaskDetailViewProps) {
  const [showObservationForm, setShowObservationForm] = useState(false);

  const getStatusColor = (state: string) => {
    switch (state.toLowerCase()) {
      case "completed":
        return "text-green-600 bg-green-100";
      case "running":
      case "in_progress":
        return "text-blue-600 bg-blue-100";
      case "failed":
      case "error":
        return "text-red-600 bg-red-100";
      case "pending":
      case "queued":
        return "text-yellow-600 bg-yellow-100";
      default:
        return "text-gray-600 bg-gray-100";
    }
  };

  const getPhaseIcon = (phase: string) => {
    switch (phase) {
      case "observation":
        return "👁️";
      case "analysis":
        return "🔍";
      case "plan":
        return "📋";
      case "decision":
        return "⚖️";
      case "execute":
        return "⚡";
      case "verify":
        return "✅";
      case "hypothesis":
        return "💡";
      case "critique":
        return "🎯";
      default:
        return "💭";
    }
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleString();
  };

  const sortedChainOfThought = [...chainOfThought].sort(
    (a, b) => new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime()
  );

  return (
    <div className="space-y-6">
      {/* Task Header */}
      <div className="bg-white rounded-lg shadow p-6">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold text-gray-900">Task Details</h2>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600 transition-colors"
          >
            <svg
              className="h-6 w-6"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M6 18L18 6M6 6l12 12"
              />
            </svg>
          </button>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div>
            <div className="flex items-center space-x-2 mb-2">
              <span className="text-sm font-medium text-gray-600">
                Task ID:
              </span>
              <code className="text-sm bg-gray-100 px-2 py-1 rounded">
                {task.taskId}
              </code>
            </div>
            <div className="flex items-center space-x-2 mb-2">
              <span className="text-sm font-medium text-gray-600">Status:</span>
              <span
                className={`px-2 py-1 rounded-full text-xs font-medium ${getStatusColor(
                  task.state
                )}`}
              >
                {task.state}
              </span>
            </div>
            <div className="mb-2">
              <span className="text-sm font-medium text-gray-600">
                Last Updated:
              </span>
              <p className="text-sm text-gray-900">
                {formatDate(task.lastUpdated)}
              </p>
            </div>
          </div>

          <div>
            {task.currentPlan && (
              <div className="mb-2">
                <span className="text-sm font-medium text-gray-600">
                  Current Plan:
                </span>
                <p className="text-sm text-gray-900">{task.currentPlan}</p>
              </div>
            )}
            {task.nextActions && task.nextActions.length > 0 && (
              <div>
                <span className="text-sm font-medium text-gray-600">
                  Next Actions:
                </span>
                <ul className="text-sm text-gray-900 list-disc list-inside">
                  {task.nextActions.map((action, index) => (
                    <li key={index}>{action}</li>
                  ))}
                </ul>
              </div>
            )}
          </div>
        </div>

        {/* CAWS and Verification Status */}
        {(task.caws || task.verification) && (
          <div className="mt-4 pt-4 border-t">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {task.caws && (
                <div>
                  <h4 className="text-sm font-medium text-gray-600 mb-2">
                    CAWS Assessment
                  </h4>
                  <div className="space-y-1">
                    <div className="flex items-center space-x-2">
                      <span
                        className={`text-sm font-medium ${
                          task.caws.passed ? "text-green-600" : "text-red-600"
                        }`}
                      >
                        {task.caws.passed ? "✓ Passed" : "✗ Failed"}
                      </span>
                    </div>
                    <p className="text-sm text-gray-600">{task.caws.verdict}</p>
                    {task.caws.remediation &&
                      task.caws.remediation.length > 0 && (
                        <div className="mt-2">
                          <p className="text-xs font-medium text-gray-600">
                            Remediation:
                          </p>
                          <ul className="text-xs text-gray-600 list-disc list-inside">
                            {task.caws.remediation.map((item, index) => (
                              <li key={index}>{item}</li>
                            ))}
                          </ul>
                        </div>
                      )}
                  </div>
                </div>
              )}

              {task.verification && (
                <div>
                  <h4 className="text-sm font-medium text-gray-600 mb-2">
                    Verification
                  </h4>
                  <div className="space-y-1">
                    <p className="text-sm text-gray-900">
                      {task.verification.verdict}
                    </p>
                    <p className="text-sm text-gray-600">
                      Confidence:{" "}
                      {(task.verification.confidence * 100).toFixed(1)}%
                    </p>
                    <div className="mt-2">
                      <p className="text-xs font-medium text-gray-600">
                        Reasoning:
                      </p>
                      <ul className="text-xs text-gray-600 list-disc list-inside">
                        {task.verification.reasoning.map((reason, index) => (
                          <li key={index}>{reason}</li>
                        ))}
                      </ul>
                    </div>
                  </div>
                </div>
              )}
            </div>
          </div>
        )}
      </div>

      {/* Progress Steps */}
      <div className="bg-white rounded-lg shadow p-6">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-gray-900">Progress</h3>
          <button
            onClick={() => setShowObservationForm(!showObservationForm)}
            className="px-3 py-1 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700 transition-colors"
          >
            Add Observation
          </button>
        </div>

        {showObservationForm && (
          <div className="mb-6">
            <ObservationForm
              onSubmit={async (message, author) => {
                await onAddObservation(message, author);
                setShowObservationForm(false);
              }}
              onCancel={() => setShowObservationForm(false)}
            />
          </div>
        )}

        {task.progress.length === 0 ? (
          <p className="text-gray-500 text-center py-8">
            No progress recorded yet
          </p>
        ) : (
          <div className="space-y-3">
            {task.progress.map((step, index) => (
              <div key={index} className="flex items-start space-x-3">
                <div className="flex-shrink-0 w-8 h-8 bg-blue-100 rounded-full flex items-center justify-center">
                  <span className="text-sm font-medium text-blue-600">
                    {index + 1}
                  </span>
                </div>
                <div className="flex-1 min-w-0">
                  <p className="text-sm text-gray-900">{step}</p>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Chain of Thought */}
      <div className="bg-white rounded-lg shadow p-6">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">
          Chain of Thought ({sortedChainOfThought.length})
        </h3>

        {sortedChainOfThought.length === 0 ? (
          <p className="text-gray-500 text-center py-8">
            No chain of thought recorded yet
          </p>
        ) : (
          <div className="space-y-4 max-h-96 overflow-y-auto">
            {sortedChainOfThought.map((entry) => (
              <div
                key={entry.id + entry.timestamp}
                className="border-l-4 border-blue-200 pl-4 py-2"
              >
                <div className="flex items-center space-x-2 mb-2">
                  <span className="text-lg">{getPhaseIcon(entry.phase)}</span>
                  <span className="text-sm font-medium text-gray-900 capitalize">
                    {entry.phase}
                  </span>
                  {entry.agentId && (
                    <span className="text-xs text-gray-500">
                      by {entry.agentId}
                    </span>
                  )}
                  {entry.confidence !== undefined && (
                    <span className="text-xs text-gray-500">
                      {Math.round(entry.confidence * 100)}% confidence
                    </span>
                  )}
                  <span className="text-xs text-gray-500 ml-auto">
                    {formatDate(entry.timestamp)}
                  </span>
                </div>

                {entry.content && (
                  <div className="text-sm text-gray-900 whitespace-pre-wrap">
                    {entry.content}
                  </div>
                )}

                {entry.redacted && (
                  <div className="mt-1 text-xs text-red-600">
                    ⚠️ This entry has been redacted for privacy
                  </div>
                )}
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
