"use client";

import ArbiterControls from "@/components/ArbiterControls";
import DashboardHeader from "@/components/DashboardHeader";
import EventViewer from "@/components/EventViewer";
import MetricsOverview from "@/components/MetricsOverview";
import SystemStatus from "@/components/SystemStatus";
import TaskManager from "@/components/TaskManager";
import { ObserverApiClient } from "@/lib/api-client";
import type {
  ObserverMetricsSnapshot,
  ObserverProgressSummary,
  ObserverStatusSummary,
} from "@/types/api";
import { useEffect, useState } from "react";

export default function Dashboard() {
  const [apiClient] = useState(() => new ObserverApiClient());
  const [status, setStatus] = useState<ObserverStatusSummary | null>(null);
  const [metrics, setMetrics] = useState<ObserverMetricsSnapshot | null>(null);
  const [progress, setProgress] = useState<ObserverProgressSummary | null>(
    null
  );
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<
    "overview" | "tasks" | "events" | "controls"
  >("overview");

  useEffect(() => {
    loadDashboardData();
    // Set up periodic refresh
    const interval = setInterval(loadDashboardData, 5000);
    return () => clearInterval(interval);
  }, []);

  const loadDashboardData = async () => {
    try {
      setError(null);
      const [statusData, metricsData, progressData] = await Promise.all([
        apiClient.getStatus(),
        apiClient.getMetrics(),
        apiClient.getProgress(),
      ]);
      setStatus(statusData);
      setMetrics(metricsData);
      setProgress(progressData);
      setLoading(false);
    } catch (err) {
      console.error("Failed to load dashboard data:", err);
      setError(
        err instanceof Error ? err.message : "Failed to load dashboard data"
      );
      setLoading(false);
    }
  };

  if (loading) {
    return (
      <div className="min-h-screen bg-gray-50 flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto mb-4"></div>
          <p className="text-gray-600">Loading Arbiter Observer Dashboard...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="min-h-screen bg-gray-50 flex items-center justify-center">
        <div className="text-center max-w-md">
          <div className="text-red-500 text-6xl mb-4">⚠️</div>
          <h1 className="text-xl font-semibold text-gray-900 mb-2">
            Connection Error
          </h1>
          <p className="text-gray-600 mb-4">{error}</p>
          <button
            onClick={loadDashboardData}
            className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors"
          >
            Retry Connection
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-50">
      <DashboardHeader activeTab={activeTab} onTabChange={setActiveTab} />

      <main className="container mx-auto px-4 py-6">
        {activeTab === "overview" && (
          <div className="space-y-6">
            <SystemStatus status={status} />
            <MetricsOverview metrics={metrics} progress={progress} />
          </div>
        )}

        {activeTab === "tasks" && <TaskManager apiClient={apiClient} />}

        {activeTab === "events" && <EventViewer apiClient={apiClient} />}

        {activeTab === "controls" && (
          <ArbiterControls apiClient={apiClient} status={status} />
        )}
      </main>
    </div>
  );
}
