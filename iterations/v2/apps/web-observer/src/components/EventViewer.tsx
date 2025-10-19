"use client";

import { ObserverApiClient } from "@/lib/api-client";
import type {
  EventFilters,
  EventListResult,
  ObserverEventPayload,
} from "@/types/api";
import { useEffect, useState } from "react";

interface EventViewerProps {
  apiClient: ObserverApiClient;
}

interface EventAnalysis {
  totalEvents: number;
  eventsBySeverity: Record<string, number>;
  eventsByType: Record<string, number>;
  eventsBySource: Record<string, number>;
  recentSpike: boolean;
  errorRate: number;
  topErrors: Array<{ type: string; count: number; message: string }>;
}

export default function EventViewer({ apiClient }: EventViewerProps) {
  const [events, setEvents] = useState<ObserverEventPayload[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [nextCursor, setNextCursor] = useState<string | undefined>();
  const [loadingMore, setLoadingMore] = useState(false);
  const [autoRefresh, setAutoRefresh] = useState(true);
  const [filters, setFilters] = useState<EventFilters>({
    limit: 50,
  });
  const [searchTerm, setSearchTerm] = useState("");
  const [showAnalysis, setShowAnalysis] = useState(false);
  const [analysis, setAnalysis] = useState<EventAnalysis | null>(null);

  useEffect(() => {
    loadEvents();
  }, [filters]);

  useEffect(() => {
    let interval: NodeJS.Timeout;
    if (autoRefresh) {
      interval = setInterval(() => {
        loadEvents(true);
      }, 5000);
    }
    return () => {
      if (interval) clearInterval(interval);
    };
  }, [autoRefresh, filters]);

  useEffect(() => {
    if (events.length > 0) {
      analyzeEvents();
    }
  }, [events]);

  const loadEvents = async (append = false) => {
    try {
      setError(null);
      if (!append) {
        setLoading(true);
      } else {
        setLoadingMore(true);
      }

      const result: EventListResult = await apiClient.getEvents(
        append && nextCursor ? { ...filters, cursor: nextCursor } : filters
      );

      if (append) {
        setEvents((prev) => [...prev, ...result.events]);
      } else {
        setEvents(result.events);
      }
      setNextCursor(result.nextCursor);
    } catch (err) {
      console.error("Failed to load events:", err);
      setError(err instanceof Error ? err.message : "Failed to load events");
    } finally {
      setLoading(false);
      setLoadingMore(false);
    }
  };

  const handleFilterChange = (
    key: keyof EventFilters,
    value: string | number | undefined
  ) => {
    setFilters((prev) => ({
      ...prev,
      [key]: value || undefined,
      cursor: undefined, // Reset pagination when filters change
    }));
  };

  const analyzeEvents = () => {
    const eventsBySeverity: Record<string, number> = {};
    const eventsByType: Record<string, number> = {};
    const eventsBySource: Record<string, number> = {};
    const errorCounts: Record<string, { count: number; message: string }> = {};

    events.forEach((event) => {
      // Count by severity
      eventsBySeverity[event.severity] =
        (eventsBySeverity[event.severity] || 0) + 1;

      // Count by type
      eventsByType[event.type] = (eventsByType[event.type] || 0) + 1;

      // Count by source
      eventsBySource[event.source] = (eventsBySource[event.source] || 0) + 1;

      // Track errors for top errors analysis
      if (event.severity === "error") {
        const errorKey = event.type;
        if (!errorCounts[errorKey]) {
          errorCounts[errorKey] = {
            count: 0,
            message: (event.metadata?.message as string) || event.type,
          };
        }
        errorCounts[errorKey].count++;
      }
    });

    const errorRate =
      events.length > 0 ? (eventsBySeverity.error || 0) / events.length : 0;

    // Check for recent spike (events in last minute)
    const oneMinuteAgo = Date.now() - 60000;
    const recentEvents = events.filter(
      (e) => new Date(e.timestamp).getTime() > oneMinuteAgo
    );
    const recentSpike = recentEvents.length > events.length * 0.3; // 30% of events in last minute

    const topErrors = Object.entries(errorCounts)
      .sort(([, a], [, b]) => b.count - a.count)
      .slice(0, 5)
      .map(([type, data]) => ({
        type,
        count: data.count,
        message: data.message,
      }));

    setAnalysis({
      totalEvents: events.length,
      eventsBySeverity,
      eventsByType,
      eventsBySource,
      recentSpike,
      errorRate,
      topErrors,
    });
  };

  const filteredEvents = events.filter((event) => {
    const matchesSearch =
      searchTerm === "" ||
      event.type.toLowerCase().includes(searchTerm.toLowerCase()) ||
      event.source.toLowerCase().includes(searchTerm.toLowerCase()) ||
      (event.metadata?.message as string)
        ?.toLowerCase()
        .includes(searchTerm.toLowerCase()) ||
      event.taskId?.toLowerCase().includes(searchTerm.toLowerCase()) ||
      event.agentId?.toLowerCase().includes(searchTerm.toLowerCase());

    const matchesSeverity =
      !filters.severity || event.severity === filters.severity;
    const matchesType = !filters.type || event.type === filters.type;
    const matchesTaskId = !filters.taskId || event.taskId === filters.taskId;

    return matchesSearch && matchesSeverity && matchesType && matchesTaskId;
  });

  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case "error":
        return "text-red-600 bg-red-100";
      case "warn":
        return "text-yellow-600 bg-yellow-100";
      case "info":
        return "text-blue-600 bg-blue-100";
      case "debug":
        return "text-gray-600 bg-gray-100";
      default:
        return "text-gray-600 bg-gray-100";
    }
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleString();
  };

  const truncateText = (text: string, maxLength: number = 100) => {
    if (text.length <= maxLength) return text;
    return text.substring(0, maxLength) + "...";
  };

  return (
    <div className="space-y-6">
      {/* Filters */}
      <div className="bg-white rounded-lg shadow p-6">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center space-x-4">
            <h2 className="text-lg font-semibold text-gray-900">Event Log</h2>
            {analysis && (
              <div className="flex items-center space-x-2">
                <span className="text-sm text-gray-500">
                  {filteredEvents.length === events.length
                    ? `${events.length} events`
                    : `${filteredEvents.length} of ${events.length} events`}
                </span>
                {analysis.recentSpike && (
                  <span className="px-2 py-1 bg-red-100 text-red-700 text-xs rounded">
                    ⚡ Spike Detected
                  </span>
                )}
              </div>
            )}
          </div>
          <div className="flex items-center space-x-4">
            <button
              onClick={() => setShowAnalysis(!showAnalysis)}
              className={`px-3 py-1 text-sm rounded-md transition-colors ${
                showAnalysis
                  ? "bg-blue-100 text-blue-700"
                  : "bg-gray-100 text-gray-700 hover:bg-gray-200"
              }`}
            >
              📊 Analysis
            </button>
            <label className="flex items-center space-x-2">
              <input
                type="checkbox"
                checked={autoRefresh}
                onChange={(e) => setAutoRefresh(e.target.checked)}
                className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
              />
              <span className="text-sm text-gray-700">Auto-refresh</span>
            </label>
            <button
              onClick={() => loadEvents()}
              className="px-3 py-1 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700 transition-colors"
            >
              Refresh
            </button>
          </div>
        </div>

        {/* Search Bar */}
        <div className="mb-4">
          <div className="relative">
            <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
              <svg
                className="h-5 w-5 text-gray-400"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
                />
              </svg>
            </div>
            <input
              type="text"
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              placeholder="Search events by type, source, message, task ID, or agent ID..."
              className="block w-full pl-10 pr-3 py-2 border border-gray-300 rounded-md leading-5 bg-white placeholder-gray-500 focus:outline-none focus:placeholder-gray-400 focus:ring-1 focus:ring-blue-500 focus:border-blue-500 text-sm"
            />
          </div>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Severity
            </label>
            <select
              value={filters.severity || ""}
              onChange={(e) => handleFilterChange("severity", e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 text-sm"
            >
              <option value="">All</option>
              <option value="debug">Debug</option>
              <option value="info">Info</option>
              <option value="warn">Warning</option>
              <option value="error">Error</option>
            </select>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Type
            </label>
            <input
              type="text"
              value={filters.type || ""}
              onChange={(e) => handleFilterChange("type", e.target.value)}
              placeholder="Event type..."
              className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 text-sm"
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Task ID
            </label>
            <input
              type="text"
              value={filters.taskId || ""}
              onChange={(e) => handleFilterChange("taskId", e.target.value)}
              placeholder="Filter by task..."
              className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 text-sm"
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Since
            </label>
            <input
              type="datetime-local"
              value={
                filters.sinceTs
                  ? new Date(filters.sinceTs).toISOString().slice(0, 16)
                  : ""
              }
              onChange={(e) =>
                handleFilterChange(
                  "sinceTs",
                  e.target.value
                    ? new Date(e.target.value).toISOString()
                    : undefined
                )
              }
              className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 text-sm"
            />
          </div>
        </div>
      </div>

      {/* Analysis Panel */}
      {showAnalysis && analysis && (
        <div className="bg-white rounded-lg shadow p-6">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">
            📊 Event Analysis
          </h3>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-6">
            {/* Severity Breakdown */}
            <div>
              <h4 className="text-sm font-medium text-gray-700 mb-3">
                Events by Severity
              </h4>
              <div className="space-y-2">
                {Object.entries(analysis.eventsBySeverity).map(
                  ([severity, count]) => (
                    <div
                      key={severity}
                      className="flex items-center justify-between"
                    >
                      <span
                        className={`text-xs px-2 py-1 rounded ${getSeverityColor(
                          severity
                        )}`}
                      >
                        {severity.toUpperCase()}
                      </span>
                      <span className="text-sm font-medium">{count}</span>
                    </div>
                  )
                )}
              </div>
            </div>

            {/* Top Sources */}
            <div>
              <h4 className="text-sm font-medium text-gray-700 mb-3">
                Top Sources
              </h4>
              <div className="space-y-2">
                {Object.entries(analysis.eventsBySource)
                  .sort(([, a], [, b]) => b - a)
                  .slice(0, 5)
                  .map(([source, count]) => (
                    <div
                      key={source}
                      className="flex items-center justify-between"
                    >
                      <span className="text-xs text-gray-600 truncate flex-1">
                        {source}
                      </span>
                      <span className="text-sm font-medium ml-2">{count}</span>
                    </div>
                  ))}
              </div>
            </div>

            {/* Error Rate */}
            <div>
              <h4 className="text-sm font-medium text-gray-700 mb-3">
                Error Rate
              </h4>
              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <span className="text-sm text-gray-600">
                    Overall Error Rate
                  </span>
                  <span
                    className={`text-sm font-medium ${
                      analysis.errorRate > 0.1
                        ? "text-red-600"
                        : analysis.errorRate > 0.05
                        ? "text-yellow-600"
                        : "text-green-600"
                    }`}
                  >
                    {(analysis.errorRate * 100).toFixed(1)}%
                  </span>
                </div>
                {analysis.recentSpike && (
                  <div className="text-xs text-red-600 bg-red-50 p-2 rounded">
                    ⚡ Event spike detected in last minute
                  </div>
                )}
              </div>
            </div>
          </div>

          {/* Top Errors */}
          {analysis.topErrors.length > 0 && (
            <div>
              <h4 className="text-sm font-medium text-gray-700 mb-3">
                Most Common Errors
              </h4>
              <div className="space-y-2">
                {analysis.topErrors.map((error, i) => (
                  <div
                    key={i}
                    className="flex items-start space-x-3 p-3 bg-red-50 rounded border border-red-200"
                  >
                    <span className="text-red-600 text-sm font-medium">
                      {error.count}x
                    </span>
                    <div className="flex-1">
                      <div className="text-sm font-medium text-red-800">
                        {error.type}
                      </div>
                      <div className="text-xs text-red-600 mt-1">
                        {truncateText(error.message, 100)}
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}

      {/* Events List */}
      <div className="bg-white rounded-lg shadow">
        {error && (
          <div className="p-4 bg-red-50 border-b border-red-200">
            <p className="text-red-800 text-sm">{error}</p>
          </div>
        )}

        {loading ? (
          <div className="p-6">
            <div className="animate-pulse space-y-4">
              {Array.from({ length: 5 }).map((_, i) => (
                <div key={i} className="border border-gray-200 rounded-lg p-4">
                  <div className="flex items-center justify-between mb-2">
                    <div className="h-4 bg-gray-200 rounded w-1/4"></div>
                    <div className="h-4 bg-gray-200 rounded w-16"></div>
                  </div>
                  <div className="h-3 bg-gray-200 rounded w-3/4 mb-2"></div>
                  <div className="h-3 bg-gray-200 rounded w-1/2"></div>
                </div>
              ))}
            </div>
          </div>
        ) : events.length === 0 ? (
          <div className="p-6 text-center">
            <svg
              className="mx-auto h-12 w-12 text-gray-400"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={1}
                d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
              />
            </svg>
            <h3 className="mt-2 text-sm font-medium text-gray-900">
              No events found
            </h3>
            <p className="mt-1 text-sm text-gray-500">
              Try adjusting your filters or check back later.
            </p>
          </div>
        ) : (
          <div className="divide-y divide-gray-200">
            {filteredEvents.map((event) => (
              <div
                key={event.id}
                className="p-4 hover:bg-gray-50 transition-colors"
              >
                <div className="flex items-start justify-between mb-2">
                  <div className="flex items-center space-x-2">
                    <span className="text-sm font-medium text-gray-900">
                      {event.type}
                    </span>
                    <span
                      className={`px-2 py-1 rounded-full text-xs font-medium ${getSeverityColor(
                        event.severity
                      )}`}
                    >
                      {event.severity.toUpperCase()}
                    </span>
                  </div>
                  <div className="flex items-center space-x-2 text-xs text-gray-500">
                    <span>{event.source}</span>
                    <span>•</span>
                    <span>{formatDate(event.timestamp)}</span>
                  </div>
                </div>

                <div className="text-sm text-gray-600 mb-2">
                  {event.metadata?.message
                    ? String(event.metadata.message)
                    : "No message"}
                </div>

                <div className="flex items-center justify-between text-xs text-gray-500">
                  <div className="flex items-center space-x-4">
                    {event.taskId && <span>Task: {event.taskId}</span>}
                    {event.agentId && <span>Agent: {event.agentId}</span>}
                    {event.traceId && (
                      <span>Trace: {truncateText(event.traceId, 20)}</span>
                    )}
                  </div>
                  <span>ID: {truncateText(event.id, 20)}</span>
                </div>
              </div>
            ))}
          </div>
        )}

        {/* Load More */}
        {nextCursor && !loading && (
          <div className="p-4 border-t border-gray-200">
            <button
              onClick={() => loadEvents(true)}
              disabled={loadingMore}
              className="w-full px-4 py-2 bg-gray-100 text-gray-700 rounded-md hover:bg-gray-200 focus:outline-none focus:ring-2 focus:ring-gray-500 focus:ring-offset-2 disabled:opacity-50 transition-colors"
            >
              {loadingMore ? "Loading..." : "Load More Events"}
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
