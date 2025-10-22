"use client";

import React, { useState, useEffect, useCallback } from "react";
import styles from "./SLODashboard.module.scss";

interface SLODefinition {
  name: string;
  description: string;
  service: string;
  metric: string;
  target: number;
  window_days: number;
  labels: Record<string, string>;
}

interface SLOStatus {
  slo_name: string;
  target_value: number;
  current_value: number;
  compliance_percentage: number;
  remaining_budget: number;
  period_start: string;
  period_end: string;
  status: "Compliant" | "AtRisk" | "Violated" | "Unknown";
  last_updated: string;
}

interface SLOMeasurement {
  slo_name: string;
  timestamp: string;
  value: number;
  sample_count: number;
  good_count: number;
  bad_count: number;
}

interface SLODashboardProps {
  refreshInterval?: number;
}

export default function SLODashboard({ refreshInterval = 60000 }: SLODashboardProps) {
  const [slos, setSlos] = useState<SLODefinition[]>([]);
  const [sloStatuses, setSloStatuses] = useState<Record<string, SLOStatus>>({});
  const [selectedSlo, setSelectedSlo] = useState<string | null>(null);
  const [measurements, setMeasurements] = useState<SLOMeasurement[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Fetch SLOs data
  const fetchSlos = useCallback(async () => {
    // try {
    //   setError(null);
    //   const response = await fetch("/api/slos");
    //   if (!response.ok) {
    //     throw new Error(`Failed to fetch SLOs: ${response.status}`);
    //   }
    //   const data = await response.json();
    //   setSlos(data.slos || []);
    // } catch (err) {
    //   setError(err instanceof Error ? err.message : "Failed to fetch SLOs");
    // }
  }, []);

  // Fetch SLO status
  const fetchSloStatus = useCallback(async (sloName: string) => {
    // try {
    //   const response = await fetch(`/api/slos/${sloName}/status`);
    //   if (!response.ok) {
    //     throw new Error(`Failed to fetch SLO status: ${response.status}`);
    //   }
    //   const data = await response.json();
    //   setSloStatuses(prev => ({
    //     ...prev,
    //     [sloName]: data.status
    //   }));
    // } catch (err) {
    //   console.error(`Failed to fetch status for ${sloName}:`, err);
    // }
  }, []);

  // Fetch SLO measurements
  const fetchMeasurements = useCallback(async (sloName: string) => {
    // try {
    //   const response = await fetch(`/api/slos/${sloName}/measurements?limit=50`);
    //   if (!response.ok) {
    //     throw new Error(`Failed to fetch measurements: ${response.status}`);
    //   }
    //   const data = await response.json();
    //   setMeasurements(data.measurements || []);
    // } catch (err) {
    //   console.error(`Failed to fetch measurements for ${sloName}:`, err);
    // }
  }, []);

  // Initial data load
  useEffect(() => {
    fetchSlos();
  }, [fetchSlos]);

  // Fetch status for all SLOs
  useEffect(() => {
    if (slos.length > 0) {
      slos.forEach(slo => fetchSloStatus(slo.name));
    }
  }, [slos, fetchSloStatus]);

  // Auto-refresh
  useEffect(() => {
    if (slos.length === 0) return;

    const interval = setInterval(() => {
      slos.forEach(slo => fetchSloStatus(slo.name));
    }, refreshInterval);

    return () => clearInterval(interval);
  }, [slos, fetchSloStatus, refreshInterval]);

  // Handle SLO selection
  const handleSloSelect = useCallback((sloName: string) => {
    setSelectedSlo(sloName);
    fetchMeasurements(sloName);
  }, [fetchMeasurements]);

  // Get status color
  const getStatusColor = (status: string) => {
    switch (status) {
      case "Compliant": return "#10b981"; // green
      case "AtRisk": return "#f59e0b"; // yellow
      case "Violated": return "#ef4444"; // red
      default: return "#6b7280"; // gray
    }
  };

  // Format percentage
  const formatPercent = (value: number) => `${(value * 100).toFixed(1)}%`;

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <h2>Service Level Objectives (SLOs)</h2>
        <p>Monitor service reliability and performance targets</p>
      </div>

      {error && (
        <div className={styles.error}>
          <span>⚠️ {error}</span>
          <button onClick={fetchSlos}>Retry</button>
        </div>
      )}

      {slos.length === 0 && !error && (
        <div className={styles.emptyState}>
          <div className={styles.emptyIcon}>📊</div>
          <h3>No SLOs Configured</h3>
          <p>
            Service Level Objectives help you monitor system reliability and performance targets.
            When connected to your API server, you'll see:
          </p>
          <ul className={styles.emptyFeatures}>
            <li>📈 <strong>Reliability metrics</strong> - Uptime percentages and error rates</li>
            <li>⚡ <strong>Performance targets</strong> - Response time and throughput goals</li>
            <li>🎯 <strong>Compliance tracking</strong> - Real-time SLO status and budget remaining</li>
            <li>📊 <strong>Historical data</strong> - Trends and measurement history</li>
          </ul>
          <div className={styles.emptyNote}>
            <span className={styles.noteIcon}>ℹ️</span>
            <span>Connect to your API server to start monitoring SLOs</span>
          </div>
        </div>
      )}

      <div className={styles.grid}>
        {slos.map(slo => {
          const status = sloStatuses[slo.name];
          const isSelected = selectedSlo === slo.name;

          return (
            <div
              key={slo.name}
              className={`${styles.sloCard} ${isSelected ? styles.selected : ''}`}
              onClick={() => handleSloSelect(slo.name)}
            >
              <div className={styles.sloHeader}>
                <h3>{slo.name}</h3>
                <div
                  className={styles.statusIndicator}
                  style={{ backgroundColor: status ? getStatusColor(status.status) : '#6b7280' }}
                />
              </div>

              <p className={styles.description}>{slo.description}</p>

              <div className={styles.metrics}>
                <div className={styles.metric}>
                  <span className={styles.label}>Target</span>
                  <span className={styles.value}>{formatPercent(slo.target)}</span>
                </div>

                {status && (
                  <>
                    <div className={styles.metric}>
                      <span className={styles.label}>Current</span>
                      <span className={styles.value}>{formatPercent(status.current_value)}</span>
                    </div>

                    <div className={styles.metric}>
                      <span className={styles.label}>Budget Left</span>
                      <span className={styles.value}>{formatPercent(status.remaining_budget)}</span>
                    </div>
                  </>
                )}
              </div>

              <div className={styles.labels}>
                <span className={styles.service}>{slo.service}</span>
                <span className={styles.metricType}>{slo.metric}</span>
              </div>

              {status && (
                <div className={styles.status}>
                  <span>Status: {status.status}</span>
                </div>
              )}
            </div>
          );
        })}
      </div>

      {selectedSlo && (
        <div className={styles.detailsPanel}>
          <h3>SLO Details: {selectedSlo}</h3>

          {measurements.length > 0 && (
            <div className={styles.measurements}>
              <h4>Recent Measurements</h4>
              <div className={styles.measurementsList}>
                {measurements.slice(0, 10).map((measurement, index) => (
                  <div key={index} className={styles.measurement}>
                    <span className={styles.timestamp}>
                      {new Date(measurement.timestamp).toLocaleString()}
                    </span>
                    <span className={styles.value}>
                      {formatPercent(measurement.value)}
                    </span>
                    <span className={styles.samples}>
                      {measurement.sample_count} samples
                    </span>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
