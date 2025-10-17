"use client";

import { ObserverApiClient } from "@/lib/api-client";
import type {
  ArbiterControlResult,
  CommandResult,
  ObserverStatusSummary,
} from "@/types/api";
import { useState } from "react";

interface ArbiterControlsProps {
  apiClient: ObserverApiClient;
  status: ObserverStatusSummary | null;
}

export default function ArbiterControls({
  apiClient,
  status,
}: ArbiterControlsProps) {
  const [starting, setStarting] = useState(false);
  const [stopping, setStopping] = useState(false);
  const [executingCommand, setExecutingCommand] = useState(false);
  const [command, setCommand] = useState("");
  const [commandResult, setCommandResult] = useState<CommandResult | null>(
    null
  );
  const [error, setError] = useState<string | null>(null);

  const handleStartArbiter = async () => {
    try {
      setStarting(true);
      setError(null);
      const result: ArbiterControlResult = await apiClient.startArbiter();
      console.log("Arbiter start result:", result);
      // Could add a success notification here
    } catch (err) {
      console.error("Failed to start arbiter:", err);
      setError(err instanceof Error ? err.message : "Failed to start arbiter");
    } finally {
      setStarting(false);
    }
  };

  const handleStopArbiter = async () => {
    try {
      setStopping(true);
      setError(null);
      const result: ArbiterControlResult = await apiClient.stopArbiter();
      console.log("Arbiter stop result:", result);
      // Could add a success notification here
    } catch (err) {
      console.error("Failed to stop arbiter:", err);
      setError(err instanceof Error ? err.message : "Failed to stop arbiter");
    } finally {
      setStopping(false);
    }
  };

  const handleExecuteCommand = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!command.trim()) return;

    try {
      setExecutingCommand(true);
      setError(null);
      const result: CommandResult = await apiClient.executeCommand(
        command.trim()
      );
      setCommandResult(result);
      setCommand("");
    } catch (err) {
      console.error("Failed to execute command:", err);
      setError(
        err instanceof Error ? err.message : "Failed to execute command"
      );
    } finally {
      setExecutingCommand(false);
    }
  };

  const getArbiterStatusDisplay = () => {
    if (!status) return "Unknown";

    // Map observer status to arbiter status
    switch (status.status) {
      case "running":
        return "Running";
      case "stopped":
        return "Stopped";
      case "degraded":
        return "Degraded";
      default:
        return "Unknown";
    }
  };

  const getStatusColor = (status: string) => {
    switch (status.toLowerCase()) {
      case "running":
        return "text-green-600 bg-green-100";
      case "stopped":
        return "text-red-600 bg-red-100";
      case "degraded":
        return "text-yellow-600 bg-yellow-100";
      default:
        return "text-gray-600 bg-gray-100";
    }
  };

  return (
    <div className="space-y-6">
      {/* Arbiter Status */}
      <div className="bg-white rounded-lg shadow p-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-4">
          Arbiter Control
        </h2>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div>
            <h3 className="text-sm font-medium text-gray-600 mb-2">Status</h3>
            <div className="flex items-center space-x-2">
              <span
                className={`px-3 py-1 rounded-full text-sm font-medium ${getStatusColor(
                  status?.status || "unknown"
                )}`}
              >
                {getArbiterStatusDisplay()}
              </span>
            </div>
            {status && (
              <div className="mt-2 text-sm text-gray-600">
                <p>Uptime: {Math.floor(status.uptimeMs / 1000 / 60)} minutes</p>
                <p>Auth: {status.authConfigured ? "Enabled" : "Disabled"}</p>
              </div>
            )}
          </div>

          <div>
            <h3 className="text-sm font-medium text-gray-600 mb-2">Actions</h3>
            <div className="flex flex-col space-y-2">
              <button
                onClick={handleStartArbiter}
                disabled={starting || status?.status === "running"}
                className="px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-green-500 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
              >
                {starting ? "Starting..." : "Start Arbiter"}
              </button>

              <button
                onClick={handleStopArbiter}
                disabled={stopping || status?.status === "stopped"}
                className="px-4 py-2 bg-red-600 text-white rounded-md hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-red-500 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
              >
                {stopping ? "Stopping..." : "Stop Arbiter"}
              </button>
            </div>
          </div>
        </div>

        {error && (
          <div className="mt-4 p-3 bg-red-50 border border-red-200 rounded-md">
            <p className="text-red-800 text-sm">{error}</p>
          </div>
        )}
      </div>

      {/* Command Execution */}
      <div className="bg-white rounded-lg shadow p-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-4">
          Command Execution
        </h2>

        <form onSubmit={handleExecuteCommand} className="space-y-4">
          <div>
            <label
              htmlFor="command"
              className="block text-sm font-medium text-gray-700 mb-1"
            >
              Management Command
            </label>
            <input
              type="text"
              id="command"
              value={command}
              onChange={(e) => setCommand(e.target.value)}
              placeholder="Enter a management command..."
              className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            />
            <p className="mt-1 text-xs text-gray-500">
              Commands are forwarded to /observer/commands endpoint for
              management operations
            </p>
          </div>

          <div className="flex justify-end">
            <button
              type="submit"
              disabled={executingCommand || !command.trim()}
              className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              {executingCommand ? "Executing..." : "Execute Command"}
            </button>
          </div>
        </form>

        {commandResult && (
          <div className="mt-4 p-4 bg-gray-50 border border-gray-200 rounded-md">
            <h4 className="text-sm font-medium text-gray-900 mb-2">
              Command Result
            </h4>
            <div className="space-y-2">
              <div className="flex items-center space-x-2">
                <span className="text-sm text-gray-600">Acknowledged:</span>
                <span
                  className={`px-2 py-1 rounded-full text-xs font-medium ${
                    commandResult.acknowledged
                      ? "text-green-600 bg-green-100"
                      : "text-red-600 bg-red-100"
                  }`}
                >
                  {commandResult.acknowledged ? "Yes" : "No"}
                </span>
              </div>
              {commandResult.note && (
                <div>
                  <span className="text-sm text-gray-600">Note:</span>
                  <p className="text-sm text-gray-900 mt-1">
                    {commandResult.note}
                  </p>
                </div>
              )}
            </div>
          </div>
        )}
      </div>

      {/* Quick Actions */}
      <div className="bg-white rounded-lg shadow p-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-4">
          Quick Actions
        </h2>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          <button
            onClick={() => setCommand("status")}
            className="p-4 border border-gray-200 rounded-lg hover:border-gray-300 hover:bg-gray-50 transition-colors text-left"
          >
            <div className="text-sm font-medium text-gray-900">
              Check Status
            </div>
            <div className="text-xs text-gray-600 mt-1">
              Get current system status
            </div>
          </button>

          <button
            onClick={() => setCommand("metrics")}
            className="p-4 border border-gray-200 rounded-lg hover:border-gray-300 hover:bg-gray-50 transition-colors text-left"
          >
            <div className="text-sm font-medium text-gray-900">
              View Metrics
            </div>
            <div className="text-xs text-gray-600 mt-1">
              Display performance metrics
            </div>
          </button>

          <button
            onClick={() => setCommand("health")}
            className="p-4 border border-gray-200 rounded-lg hover:border-gray-300 hover:bg-gray-50 transition-colors text-left"
          >
            <div className="text-sm font-medium text-gray-900">
              Health Check
            </div>
            <div className="text-xs text-gray-600 mt-1">
              Run system health diagnostics
            </div>
          </button>

          <button
            onClick={() => setCommand("tasks")}
            className="p-4 border border-gray-200 rounded-lg hover:border-gray-300 hover:bg-gray-50 transition-colors text-left"
          >
            <div className="text-sm font-medium text-gray-900">List Tasks</div>
            <div className="text-xs text-gray-600 mt-1">Show active tasks</div>
          </button>

          <button
            onClick={() => setCommand("clear")}
            className="p-4 border border-gray-200 rounded-lg hover:border-gray-300 hover:bg-gray-50 transition-colors text-left"
          >
            <div className="text-sm font-medium text-gray-900">Clear Logs</div>
            <div className="text-xs text-gray-600 mt-1">Clear event logs</div>
          </button>

          <button
            onClick={() => setCommand("restart")}
            className="p-4 border border-gray-200 rounded-lg hover:border-gray-300 hover:bg-gray-50 transition-colors text-left"
          >
            <div className="text-sm font-medium text-gray-900">
              Restart Services
            </div>
            <div className="text-xs text-gray-600 mt-1">
              Restart background services
            </div>
          </button>
        </div>
      </div>
    </div>
  );
}
