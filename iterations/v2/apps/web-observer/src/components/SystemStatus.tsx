import type { ObserverStatusSummary } from "@/types/api";

interface SystemStatusProps {
  status: ObserverStatusSummary | null;
}

export default function SystemStatus({ status }: SystemStatusProps) {
  if (!status) {
    return (
      <div className="bg-white rounded-lg shadow p-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-4">
          System Status
        </h2>
        <div className="animate-pulse">
          <div className="h-4 bg-gray-200 rounded w-1/4 mb-2"></div>
          <div className="h-4 bg-gray-200 rounded w-1/2"></div>
        </div>
      </div>
    );
  }

  const getStatusColor = (status: string) => {
    switch (status) {
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

  const formatUptime = (ms: number) => {
    const seconds = Math.floor(ms / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    const days = Math.floor(hours / 24);

    if (days > 0) return `${days}d ${hours % 24}h`;
    if (hours > 0) return `${hours}h ${minutes % 60}m`;
    if (minutes > 0) return `${minutes}m ${seconds % 60}s`;
    return `${seconds}s`;
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleString();
  };

  return (
    <div className="bg-white rounded-lg shadow p-6">
      <h2 className="text-lg font-semibold text-gray-900 mb-4">
        System Status
      </h2>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <div className="space-y-2">
          <div className="flex items-center space-x-2">
            <span className="text-sm font-medium text-gray-600">Status</span>
            <span
              className={`px-2 py-1 rounded-full text-xs font-medium ${getStatusColor(
                status.status
              )}`}
            >
              {status.status.toUpperCase()}
            </span>
          </div>
          <p className="text-sm text-gray-900">
            Started: {formatDate(status.startedAt)}
          </p>
          <p className="text-sm text-gray-900">
            Uptime: {formatUptime(status.uptimeMs)}
          </p>
        </div>

        <div className="space-y-2">
          <h3 className="text-sm font-medium text-gray-600">Queue</h3>
          <p className="text-sm text-gray-900">Depth: {status.queueDepth}</p>
          <p className="text-sm text-gray-900">Max: {status.maxQueueSize}</p>
          <div className="w-full bg-gray-200 rounded-full h-2">
            <div
              className="bg-blue-600 h-2 rounded-full"
              style={{
                width: `${Math.min(
                  (status.queueDepth / status.maxQueueSize) * 100,
                  100
                )}%`,
              }}
            />
          </div>
        </div>

        <div className="space-y-2">
          <h3 className="text-sm font-medium text-gray-600">Observer</h3>
          <p className="text-sm text-gray-900">
            Degraded: {status.observerDegraded ? "Yes" : "No"}
          </p>
          <p className="text-sm text-gray-900">
            Auth: {status.authConfigured ? "Configured" : "Not configured"}
          </p>
          <p className="text-sm text-gray-900">
            Backpressure: {status.backpressureEvents}
          </p>
        </div>

        <div className="space-y-2">
          <h3 className="text-sm font-medium text-gray-600">Storage</h3>
          <p className="text-sm text-gray-900">
            Active File: {status.activeFile || "N/A"}
          </p>
          <p className="text-sm text-gray-900">
            Last Flush: {formatUptime(Date.now() - status.lastFlushMs)} ago
          </p>
        </div>
      </div>
    </div>
  );
}
