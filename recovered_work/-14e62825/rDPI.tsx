"use client";

import { ObserverApiClient } from "@/lib/api-client";
import { useEffect, useState } from "react";

interface DatabaseAuditPanelProps {
  apiClient: ObserverApiClient;
}

interface DatabaseTableInfo {
  tableName: string;
  rowCount: number;
  sizeBytes: number;
  lastUpdated: string;
  indexes: number;
  constraints: number;
}

interface DatabaseAuditInfo {
  connectionStatus: "connected" | "disconnected" | "error";
  databaseName: string;
  totalTables: number;
  totalRows: number;
  totalSizeBytes: number;
  tables: DatabaseTableInfo[];
  recentActivity: Array<{
    timestamp: string;
    operation: string;
    table: string;
    details: string;
  }>;
  performanceMetrics: {
    avgQueryTime: number;
    activeConnections: number;
    cacheHitRatio: number;
    slowQueries: number;
  };
}

export default function DatabaseAuditPanel({ apiClient: _apiClient }: DatabaseAuditPanelProps) {
  const [auditInfo, setAuditInfo] = useState<DatabaseAuditInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [expanded, setExpanded] = useState(false);
  const [selectedTable, setSelectedTable] = useState<string | null>(null);

  useEffect(() => {
    loadDatabaseAudit();
    // Refresh every 30 seconds
    const interval = setInterval(loadDatabaseAudit, 30000);
    return () => clearInterval(interval);
  }, []);

  const loadDatabaseAudit = async () => {
    try {
      // For now, we'll simulate database audit data since we don't have direct DB access
      // In a real implementation, this would query the database directly
      const mockAuditInfo: DatabaseAuditInfo = {
        connectionStatus: "connected",
        databaseName: "agent_agency_v2",
        totalTables: 8,
        totalRows: 1247,
        totalSizeBytes: 2048576, // ~2MB
        tables: [
          {
            tableName: "agent_profiles",
            rowCount: 4,
            sizeBytes: 8192,
            lastUpdated: "2025-10-19T22:04:08.000Z",
            indexes: 3,
            constraints: 2,
          },
          {
            tableName: "agent_capabilities",
            rowCount: 12,
            sizeBytes: 16384,
            lastUpdated: "2025-10-19T22:04:08.000Z",
            indexes: 2,
            constraints: 1,
          },
          {
            tableName: "task_queue",
            rowCount: 0,
            sizeBytes: 4096,
            lastUpdated: "2025-10-19T22:04:08.000Z",
            indexes: 1,
            constraints: 3,
          },
          {
            tableName: "performance_events",
            rowCount: 156,
            sizeBytes: 524288,
            lastUpdated: "2025-10-19T22:04:21.000Z",
            indexes: 4,
            constraints: 2,
          },
          {
            tableName: "verification_requests",
            rowCount: 23,
            sizeBytes: 32768,
            lastUpdated: "2025-10-19T22:04:15.000Z",
            indexes: 3,
            constraints: 2,
          },
          {
            tableName: "knowledge_queries",
            rowCount: 45,
            sizeBytes: 65536,
            lastUpdated: "2025-10-19T22:04:18.000Z",
            indexes: 3,
            constraints: 1,
          },
          {
            tableName: "learning_episodes",
            rowCount: 89,
            sizeBytes: 131072,
            lastUpdated: "2025-10-19T22:04:20.000Z",
            indexes: 2,
            constraints: 1,
          },
          {
            tableName: "audit_logs",
            rowCount: 918,
            sizeBytes: 1048576,
            lastUpdated: "2025-10-19T22:04:21.000Z",
            indexes: 2,
            constraints: 1,
          },
        ],
        recentActivity: [
          {
            timestamp: "2025-10-19T22:04:21.000Z",
            operation: "INSERT",
            table: "performance_events",
            details: "Task execution complete event",
          },
          {
            timestamp: "2025-10-19T22:04:20.000Z",
            operation: "INSERT",
            table: "learning_episodes",
            details: "New learning episode recorded",
          },
          {
            timestamp: "2025-10-19T22:04:18.000Z",
            operation: "INSERT",
            table: "knowledge_queries",
            details: "Knowledge query processed",
          },
          {
            timestamp: "2025-10-19T22:04:15.000Z",
            operation: "INSERT",
            table: "verification_requests",
            details: "Verification request created",
          },
          {
            timestamp: "2025-10-19T22:04:08.000Z",
            operation: "INSERT",
            table: "agent_profiles",
            details: "Agent registry initialization",
          },
        ],
        performanceMetrics: {
          avgQueryTime: 12.5,
          activeConnections: 3,
          cacheHitRatio: 94.2,
          slowQueries: 2,
        },
      };

      setAuditInfo(mockAuditInfo);
      setLoading(false);
    } catch (err) {
      console.error("Failed to load database audit:", err);
      setLoading(false);
    }
  };

  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
  };

  const getTableHealthColor = (table: DatabaseTableInfo): string => {
    if (table.rowCount === 0) return "text-gray-500";
    if (table.sizeBytes > 1000000) return "text-orange-600"; // > 1MB
    if (table.rowCount > 1000) return "text-yellow-600";
    return "text-green-600";
  };

  const getConnectionStatusColor = (status: string): string => {
    switch (status) {
      case "connected":
        return "text-green-600 bg-green-100";
      case "disconnected":
        return "text-red-600 bg-red-100";
      case "error":
        return "text-red-600 bg-red-100";
      default:
        return "text-gray-600 bg-gray-100";
    }
  };

  if (loading) {
    return (
      <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
        <div className="flex items-center">
          <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-blue-600 mr-2"></div>
          <span className="text-blue-800 text-sm">Loading database audit...</span>
        </div>
      </div>
    );
  }

  if (!auditInfo) {
    return null;
  }

  return (
    <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center">
          <svg
            className="h-5 w-5 text-blue-600 mr-2"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4m0 5c0 2.21-3.582 4-8 4s-8-1.79-8-4"
            />
          </svg>
          <h3 className="text-blue-800 font-medium">
            Database Audit - {auditInfo.databaseName}
          </h3>
          <span
            className={`ml-2 px-2 py-1 rounded text-xs font-medium ${getConnectionStatusColor(
              auditInfo.connectionStatus
            )}`}
          >
            {auditInfo.connectionStatus.toUpperCase()}
          </span>
        </div>
        <button
          onClick={() => setExpanded(!expanded)}
          className="text-blue-600 hover:text-blue-800 text-sm font-medium"
        >
          {expanded ? "Hide Details" : "Show Details"}
        </button>
      </div>

      {/* Quick Stats */}
      <div className="mt-3 grid grid-cols-4 gap-4 text-sm">
        <div className="text-center">
          <div className="font-semibold text-blue-900">{auditInfo.totalTables}</div>
          <div className="text-blue-700">Tables</div>
        </div>
        <div className="text-center">
          <div className="font-semibold text-blue-900">{auditInfo.totalRows.toLocaleString()}</div>
          <div className="text-blue-700">Total Rows</div>
        </div>
        <div className="text-center">
          <div className="font-semibold text-blue-900">{formatBytes(auditInfo.totalSizeBytes)}</div>
          <div className="text-blue-700">Total Size</div>
        </div>
        <div className="text-center">
          <div className="font-semibold text-blue-900">{auditInfo.performanceMetrics.activeConnections}</div>
          <div className="text-blue-700">Connections</div>
        </div>
      </div>

      {expanded && (
        <div className="mt-4 space-y-4">
          {/* Database Tables */}
          <div>
            <h4 className="text-blue-800 font-medium mb-2">Database Tables</h4>
            <div className="bg-white rounded border overflow-hidden">
              <div className="overflow-x-auto">
                <table className="w-full text-sm">
                  <thead className="bg-gray-50">
                    <tr>
                      <th className="px-3 py-2 text-left font-medium text-gray-700">Table</th>
                      <th className="px-3 py-2 text-left font-medium text-gray-700">Rows</th>
                      <th className="px-3 py-2 text-left font-medium text-gray-700">Size</th>
                      <th className="px-3 py-2 text-left font-medium text-gray-700">Indexes</th>
                      <th className="px-3 py-2 text-left font-medium text-gray-700">Last Updated</th>
                      <th className="px-3 py-2 text-left font-medium text-gray-700">Health</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-gray-200">
                    {auditInfo.tables.map((table) => (
                      <tr
                        key={table.tableName}
                        className="hover:bg-gray-50 cursor-pointer"
                        onClick={() => setSelectedTable(selectedTable === table.tableName ? null : table.tableName)}
                      >
                        <td className="px-3 py-2 font-mono text-gray-900">{table.tableName}</td>
                        <td className="px-3 py-2 text-gray-700">{table.rowCount.toLocaleString()}</td>
                        <td className="px-3 py-2 text-gray-700">{formatBytes(table.sizeBytes)}</td>
                        <td className="px-3 py-2 text-gray-700">{table.indexes}</td>
                        <td className="px-3 py-2 text-gray-700">
                          {new Date(table.lastUpdated).toLocaleString()}
                        </td>
                        <td className="px-3 py-2">
                          <span className={`text-xs font-medium ${getTableHealthColor(table)}`}>
                            {table.rowCount === 0 ? "Empty" : 
                             table.sizeBytes > 1000000 ? "Large" :
                             table.rowCount > 1000 ? "Active" : "Healthy"}
                          </span>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          </div>

          {/* Performance Metrics */}
          <div>
            <h4 className="text-blue-800 font-medium mb-2">Performance Metrics</h4>
            <div className="grid grid-cols-2 gap-4 text-sm">
              <div className="bg-white p-3 rounded border">
                <div className="text-gray-600">Avg Query Time</div>
                <div className="font-medium text-lg">{auditInfo.performanceMetrics.avgQueryTime}ms</div>
              </div>
              <div className="bg-white p-3 rounded border">
                <div className="text-gray-600">Cache Hit Ratio</div>
                <div className="font-medium text-lg">{auditInfo.performanceMetrics.cacheHitRatio}%</div>
              </div>
              <div className="bg-white p-3 rounded border">
                <div className="text-gray-600">Active Connections</div>
                <div className="font-medium text-lg">{auditInfo.performanceMetrics.activeConnections}</div>
              </div>
              <div className="bg-white p-3 rounded border">
                <div className="text-gray-600">Slow Queries</div>
                <div className="font-medium text-lg">{auditInfo.performanceMetrics.slowQueries}</div>
              </div>
            </div>
          </div>

          {/* Recent Activity */}
          <div>
            <h4 className="text-blue-800 font-medium mb-2">Recent Database Activity</h4>
            <div className="space-y-2">
              {auditInfo.recentActivity.map((activity, i) => (
                <div key={i} className="bg-white p-3 rounded border text-sm">
                  <div className="flex items-center justify-between mb-1">
                    <span className="font-medium text-gray-900">{activity.operation}</span>
                    <span className="text-gray-500 text-xs">{activity.timestamp}</span>
                  </div>
                  <div className="text-gray-700">
                    <span className="font-mono text-blue-600">{activity.table}</span> - {activity.details}
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* Database Health Assessment */}
          <div>
            <h4 className="text-blue-800 font-medium mb-2">Database Health Assessment</h4>
            <div className="space-y-2 text-sm">
              {auditInfo.performanceMetrics.cacheHitRatio > 90 && (
                <div className="text-green-700 bg-green-100 p-2 rounded">
                  ✅ Cache performance is excellent ({auditInfo.performanceMetrics.cacheHitRatio}% hit ratio)
                </div>
              )}
              {auditInfo.performanceMetrics.avgQueryTime < 50 && (
                <div className="text-green-700 bg-green-100 p-2 rounded">
                  ✅ Query performance is good (avg {auditInfo.performanceMetrics.avgQueryTime}ms)
                </div>
              )}
              {auditInfo.performanceMetrics.slowQueries > 0 && (
                <div className="text-yellow-700 bg-yellow-100 p-2 rounded">
                  ⚠️ {auditInfo.performanceMetrics.slowQueries} slow queries detected
                </div>
              )}
              {auditInfo.tables.some(t => t.sizeBytes > 1000000) && (
                <div className="text-orange-700 bg-orange-100 p-2 rounded">
                  📊 Large tables detected - consider archiving old data
                </div>
              )}
              {auditInfo.totalRows > 10000 && (
                <div className="text-blue-700 bg-blue-100 p-2 rounded">
                  📈 Database is actively used ({auditInfo.totalRows.toLocaleString()} total rows)
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
