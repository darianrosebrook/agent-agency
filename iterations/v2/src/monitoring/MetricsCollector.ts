/**
 * Metrics Collector - System Resource Monitoring
 *
 * Collects system-level metrics including CPU, memory, disk, and network usage.
 * Uses Node.js built-in APIs for cross-platform compatibility.
 *
 * @author @darianrosebrook
 */

import { cpus, freemem, loadavg, totalmem } from "os";
import { PerformanceTrackerDatabaseClient } from "../database/PerformanceTrackerDatabaseClient.js";
import { SystemMetrics } from "./types.js";

export class MetricsCollector {
  private previousNetworkStats: Map<
    string,
    { bytesIn: number; bytesOut: number }
  > = new Map();
  private lastCollectionTime: number = Date.now();
  private databaseClient?: PerformanceTrackerDatabaseClient;

  /**
   * Initialize the metrics collector with database client
   */
  async initialize(
    databaseClient?: PerformanceTrackerDatabaseClient
  ): Promise<void> {
    this.databaseClient = databaseClient;
  }

  /**
   * Collect comprehensive system metrics
   */
  async collectSystemMetrics(): Promise<SystemMetrics> {
    const now = Date.now();
    const timeDiff = (now - this.lastCollectionTime) / 1000; // seconds

    // CPU usage
    const cpuUsage = await this.getCpuUsage(timeDiff);

    // Memory usage
    const memInfo = this.getMemoryInfo();

    // TODO: Implement comprehensive disk monitoring and analytics
    //       Currently returns basic disk info; should implement comprehensive disk monitoring with performance metrics, health monitoring, and predictive analytics.
    //
    // COMPLETION CHECKLIST:
    // [ ] Primary functionality implemented
    // [ ] Monitor all mounted filesystems with detailed usage statistics
    // [ ] Implement disk I/O performance metrics (read/write throughput, latency, IOPS)
    // [ ] Add disk health monitoring (SMART attributes, error rates, temperature)
    // [ ] Support disk space forecasting and capacity planning alerts
    // [ ] Implement disk usage trend analysis and anomaly detection
    // [ ] Add disk partitioning and volume management monitoring
    // [ ] Support RAID and storage array monitoring and health checks
    // [ ] Implement disk performance benchmarking and optimization recommendations
    // [ ] API/data structures defined & stable
    // [ ] Error handling + validation aligned with error taxonomy
    // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
    // [ ] Integration tests for external systems/contracts
    // [ ] Documentation: public API + system behavior
    // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
    // [ ] Security posture reviewed (inputs, authz, sandboxing)
    // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
    // [ ] Configurability and feature flags defined if relevant
    // [ ] Failure-mode cards documented (degradation paths)
    //
    // ACCEPTANCE CRITERIA:
    // - All mounted filesystems are monitored with accurate usage statistics
    // - Disk I/O metrics provide real-time throughput and latency data
    // - SMART attributes are monitored for predictive disk failure detection
    // - Disk space forecasting provides accurate capacity planning
    // - Trend analysis identifies unusual disk usage patterns
    // - RAID and storage arrays are properly monitored and reported
    // - Performance benchmarking provides actionable optimization insights
    // - Alerting triggers appropriately for disk health and capacity issues
    // - Integration tests validate disk monitoring across different storage types
    //
    // DEPENDENCIES:
    // - System disk monitoring APIs (Required)
    // - SMART attribute reading capabilities (Required)
    // - Storage array management interfaces (Optional)
    // - Forecasting and trend analysis algorithms (Required)
    // - Alerting and notification system (Required)
    //
    // ESTIMATED EFFORT: 16-20 hours (medium confidence)
    // PRIORITY: High
    // BLOCKING: No
    //
    // GOVERNANCE:
    // - CAWS Tier: 1 (infrastructure monitoring)
    // - Change Budget: ~350 LOC
    // - Reviewer Requirements: Storage systems and infrastructure monitoring expertise
    const diskInfo = await this.getDiskInfo();

    // Network I/O
    const networkIO = await this.getNetworkIO(timeDiff);

    // Load average
    const loadAverage = loadavg();

    this.lastCollectionTime = now;

    const metrics: SystemMetrics = {
      cpuUsage,
      memoryUsage: memInfo.usage,
      availableMemoryMB: memInfo.availableMB,
      totalMemoryMB: memInfo.totalMB,
      diskUsage: diskInfo.usage,
      availableDiskGB: diskInfo.availableGB,
      networkIO,
      loadAverage: [loadAverage[0], loadAverage[1], loadAverage[2]],
      timestamp: new Date(),
    };

    // Store metrics in database if client is available
    if (this.databaseClient) {
      try {
        await this.databaseClient.storeSystemHealthMetrics({
          timestamp: metrics.timestamp.toISOString(),
          cpuUsage: metrics.cpuUsage,
          memoryUsage: metrics.memoryUsage,
          activeConnections: 0, // Would need to be passed from connection pool
          queueDepth: 0, // Would need to be passed from task queue
          errorRate: 0, // Would need to be calculated from recent errors
          responseTimeMs: 0, // Would need to be calculated from recent requests
        });
      } catch (error) {
        console.warn("Failed to store system metrics in database:", error);
        // Continue execution - database failure shouldn't break metrics collection
      }
    }

    return metrics;
  }

  /**
   * Calculate CPU usage percentage
   */
  private async getCpuUsage(_timeDiff: number): Promise<number> {
    try {
      const cpuData = cpus();
      let totalIdle = 0;
      let totalTick = 0;

      for (const cpu of cpuData) {
        for (const type in cpu.times) {
          totalTick += (cpu.times as any)[type];
        }
        totalIdle += cpu.times.idle;
      }

      const _idle = totalIdle / cpus.length;
      const _total = totalTick / cpus.length;

      // TODO: Implement accurate CPU usage tracking with historical data
      //       Currently uses basic load average; should implement comprehensive CPU monitoring with per-core tracking, historical analysis, and performance optimization.
      //
      // COMPLETION CHECKLIST:
      // [ ] Primary functionality implemented
      // [ ] Track CPU ticks across collection intervals for precise usage calculation
      // [ ] Implement per-core CPU usage monitoring and aggregation
      // [ ] Add CPU usage trend analysis and forecasting capabilities
      // [ ] Support CPU usage anomaly detection and alerting
      // [ ] Implement CPU performance profiling and bottleneck identification
      // [ ] Add CPU usage correlation with application performance metrics
      // [ ] Support CPU usage attribution by process and thread
      // [ ] Implement CPU usage optimization recommendations
      // [ ] API/data structures defined & stable
      // [ ] Error handling + validation aligned with error taxonomy
      // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
      // [ ] Integration tests for external systems/contracts
      // [ ] Documentation: public API + system behavior
      // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
      // [ ] Security posture reviewed (inputs, authz, sandboxing)
      // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
      // [ ] Configurability and feature flags defined if relevant
      // [ ] Failure-mode cards documented (degradation paths)
      //
      // ACCEPTANCE CRITERIA:
      // - CPU usage is calculated accurately using tick differences
      // - Per-core monitoring provides detailed utilization breakdown
      // - Historical data enables trend analysis and forecasting
      // - Anomaly detection identifies unusual CPU usage patterns
      // - Performance profiling identifies CPU bottlenecks
      // - Process attribution shows CPU usage by application components
      // - Correlation analysis links CPU usage to application performance
      // - Optimization recommendations provide actionable improvements
      // - Alerting triggers appropriately for CPU usage thresholds
      // - Integration tests validate CPU monitoring across different workloads
      //
      // DEPENDENCIES:
      // - System CPU monitoring APIs (Required)
      // - Process attribution capabilities (Required)
      // - Historical data storage (Required)
      // - Statistical analysis for trends and forecasting (Required)
      // - Anomaly detection algorithms (Optional)
      //
      // ESTIMATED EFFORT: 14-18 hours (medium confidence)
      // PRIORITY: High
      // BLOCKING: No
      //
      // GOVERNANCE:
      // - CAWS Tier: 1 (infrastructure monitoring)
      // - Change Budget: ~300 LOC
      // - Reviewer Requirements: Systems monitoring and performance analysis expertise
      const loadAvg = loadavg()[0];
      const cpuCount = cpus.length;

      // TODO: Replace load average estimation with actual CPU usage calculation
      //       Currently uses load average approximation; should implement precise CPU usage calculation using tick differences and comprehensive time accounting.
      //
      // COMPLETION CHECKLIST:
      // [ ] Primary functionality implemented
      // [ ] Use CPU tick differences between collection intervals for accuracy
      // [ ] Implement proper CPU time accounting (user, system, idle, iowait, steal, guest)
      // [ ] Support CPU usage breakdown by core and NUMA node
      // [ ] Add CPU frequency and temperature monitoring capabilities
      // [ ] Implement CPU usage normalization and cross-system comparison
      // [ ] Add CPU usage historical trending and baseline comparison
      // [ ] Support CPU usage aggregation at different levels (core, socket, system)
      // [ ] Implement CPU usage alerting and threshold monitoring
      // [ ] API/data structures defined & stable
      // [ ] Error handling + validation aligned with error taxonomy
      // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
      // [ ] Integration tests for external systems/contracts
      // [ ] Documentation: public API + system behavior
      // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
      // [ ] Security posture reviewed (inputs, authz, sandboxing)
      // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
      // [ ] Configurability and feature flags defined if relevant
      // [ ] Failure-mode cards documented (degradation paths)
      //
      // ACCEPTANCE CRITERIA:
      // - CPU usage calculation uses actual tick differences for precision
      // - All CPU time states are properly accounted (user, system, idle, etc.)
      // - Per-core breakdown provides detailed utilization analysis
      // - CPU frequency and temperature monitoring works across platforms
      // - Usage normalization enables meaningful cross-system comparisons
      // - Historical trending identifies CPU usage patterns and baselines
      // - Aggregation supports different levels of CPU monitoring granularity
      // - Alerting triggers appropriately for CPU usage anomalies
      // - Performance meets SLA for real-time CPU monitoring
      // - Integration tests validate CPU monitoring across different hardware
      //
      // DEPENDENCIES:
      // - System CPU tick monitoring APIs (Required)
      // - CPU frequency and temperature sensor access (Optional)
      // - Cross-platform CPU monitoring compatibility (Required)
      // - Statistical analysis for trending and normalization (Optional)
      // - Alerting system for CPU usage thresholds (Optional)
      //
      // ESTIMATED EFFORT: 10-14 hours (medium confidence)
      // PRIORITY: High
      // BLOCKING: No
      //
      // GOVERNANCE:
      // - CAWS Tier: 1 (infrastructure monitoring)
      // - Change Budget: ~250 LOC
      // - Reviewer Requirements: Systems monitoring and CPU performance expertise
      const estimatedUsage = Math.min(100, (loadAvg / cpuCount) * 100);

      return Math.round(estimatedUsage * 100) / 100;
    } catch (error) {
      console.warn("Failed to collect CPU metrics:", error);
      return 50; // Neutral fallback
    }
  }

  /**
   * Get memory information
   */
  private getMemoryInfo(): {
    usage: number;
    availableMB: number;
    totalMB: number;
  } {
    try {
      const free = freemem();
      const total = totalmem();
      const used = total - free;

      const usage = (used / total) * 100;
      const availableMB = Math.round(free / (1024 * 1024));
      const totalMB = Math.round(total / (1024 * 1024));

      return {
        usage: Math.round(usage * 100) / 100,
        availableMB,
        totalMB,
      };
    } catch (error) {
      console.warn("Failed to collect memory metrics:", error);
      return { usage: 50, availableMB: 1024, totalMB: 2048 }; // Neutral fallback
    }
  }

  /**
   * TODO: Implement comprehensive disk usage monitoring
   *       Currently returns placeholder disk info; should implement comprehensive disk monitoring with cross-platform support, performance metrics, and predictive analytics.
   *
   * COMPLETION CHECKLIST:
   * [ ] Primary functionality implemented
   * [ ] Use cross-platform disk usage libraries (diskusage, df, GetDiskFreeSpaceEx, statvfs)
   * [ ] Monitor all mounted filesystems with detailed capacity and usage statistics
   * [ ] Implement disk space forecasting and low-space alerting capabilities
   * [ ] Support disk I/O performance metrics and throughput monitoring
   * [ ] Add disk health monitoring (bad sectors, reallocation counts, SMART attributes)
   * [ ] Implement disk usage trend analysis and growth prediction algorithms
   * [ ] Support disk quota monitoring and enforcement mechanisms
   * [ ] Add disk usage visualization and reporting capabilities
   * [ ] API/data structures defined & stable
   * [ ] Error handling + validation aligned with error taxonomy
   * [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
   * [ ] Integration tests for external systems/contracts
   * [ ] Documentation: public API + system behavior
   * [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
   * [ ] Security posture reviewed (inputs, authz, sandboxing)
   * [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
   * [ ] Configurability and feature flags defined if relevant
   * [ ] Failure-mode cards documented (degradation paths)
   *
   * ACCEPTANCE CRITERIA:
   * - All mounted filesystems are monitored with accurate usage statistics
   * - Cross-platform compatibility works on Windows, Linux, and macOS
   * - Disk space forecasting provides accurate capacity planning
   * - Low-space alerting triggers at configurable thresholds
   * - Disk I/O performance metrics are collected and reported
   * - Disk health monitoring detects and reports potential issues
   * - Trend analysis identifies unusual disk usage patterns
   * - Disk quota monitoring enforces storage limits appropriately
   * - Visualization provides clear disk usage status dashboards
   * - Integration tests validate disk monitoring across different filesystems
   *
   * DEPENDENCIES:
   * - Cross-platform disk monitoring libraries (Required)
   * - File system access permissions (Required)
   * - Forecasting and trend analysis algorithms (Required)
   * - Alerting and notification system (Required)
   * - Disk health monitoring capabilities (Optional)
   * - Quota management interfaces (Optional)
   *
   * ESTIMATED EFFORT: 14-18 hours (medium confidence)
   * PRIORITY: High
   * BLOCKING: No
   *
   * GOVERNANCE:
   * - CAWS Tier: 1 (infrastructure monitoring)
   * - Change Budget: ~300 LOC
   * - Reviewer Requirements: Storage systems and filesystem monitoring expertise
   */
  private async getDiskInfo(): Promise<{ usage: number; availableGB: number }> {
    try {
      // TODO: Replace mock disk monitoring with actual filesystem queries
      //       Currently uses memory usage as proxy for disk usage; should implement real filesystem monitoring with cross-platform support and comprehensive disk analytics.
      //
      // COMPLETION CHECKLIST:
      // [ ] Primary functionality implemented
      // [ ] Use Node.js fs.statvfs() or cross-platform disk libraries (check-disk-space, diskusage)
      // [ ] Implement proper disk space calculation for all mounted volumes
      // [ ] Support disk usage monitoring for different filesystem types (ext4, NTFS, APFS, etc.)
      // [ ] Add disk space allocation tracking and fragmentation analysis
      // [ ] Implement disk I/O statistics collection (read/write operations, throughput)
      // [ ] Add disk health monitoring (SMART attributes, error rates)
      // [ ] Support disk usage forecasting and capacity planning
      // [ ] Implement disk usage alerting and threshold monitoring
      // [ ] API/data structures defined & stable
      // [ ] Error handling + validation aligned with error taxonomy
      // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
      // [ ] Integration tests for external systems/contracts
      // [ ] Documentation: public API + system behavior
      // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
      // [ ] Security posture reviewed (inputs, authz, sandboxing)
      // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
      // [ ] Configurability and feature flags defined if relevant
      // [ ] Failure-mode cards documented (degradation paths)
      //
      // ACCEPTANCE CRITERIA:
      // - Disk space calculations use actual filesystem APIs, not memory proxies
      // - All mounted volumes are properly detected and monitored
      // - Different filesystem types are supported with appropriate calculations
      // - Disk fragmentation analysis provides actionable insights
      // - I/O statistics collection works across different disk types
      // - Disk health monitoring integrates with SMART technology
      // - Forecasting algorithms provide accurate capacity planning
      // - Alerting triggers appropriately for disk usage thresholds
      // - Cross-platform compatibility works on all supported operating systems
      // - Integration tests validate disk monitoring against system utilities
      //
      // DEPENDENCIES:
      // - Cross-platform filesystem monitoring libraries (Required)
      // - Disk I/O statistics collection APIs (Required)
      // - SMART attribute reading capabilities (Optional)
      // - Forecasting and statistical analysis libraries (Required)
      // - Alerting and notification system (Required)
      //
      // ESTIMATED EFFORT: 12-16 hours (medium confidence)
      // PRIORITY: High
      // BLOCKING: No
      //
      // GOVERNANCE:
      // - CAWS Tier: 1 (infrastructure monitoring)
      // - Change Budget: ~250 LOC
      // - Reviewer Requirements: Storage systems and filesystem monitoring expertise
      // For now, return mock values based on available memory (as a proxy)
      const memUsage = this.getMemoryInfo();

      // Mock disk usage inversely related to memory usage
      const diskUsage = Math.max(10, Math.min(90, 100 - memUsage.usage));
      const availableGB = Math.round((100 - diskUsage) * 10); // Mock 100GB total

      return {
        usage: Math.round(diskUsage * 100) / 100,
        availableGB,
      };
    } catch (error) {
      console.warn("Failed to collect disk metrics:", error);
      return { usage: 50, availableGB: 50 }; // Neutral fallback
    }
  }

  /**
   * Get network I/O statistics
   */
  private async getNetworkIO(
    _timeDiff: number
  ): Promise<{ bytesInPerSecond: number; bytesOutPerSecond: number }> {
    try {
      // TODO: Implement comprehensive network I/O monitoring
      //       Currently returns mock network statistics; should implement comprehensive network monitoring with cross-platform support, performance metrics, and predictive analytics.
      //
      // COMPLETION CHECKLIST:
      // [ ] Primary functionality implemented
      // [ ] Use system-specific network monitoring APIs (GetIfTable, netstat, /proc/net/dev, NetworkInformation API)
      // [ ] Monitor all network interfaces with detailed traffic statistics
      // [ ] Implement network latency and packet loss measurement
      // [ ] Add network utilization thresholds and alerting capabilities
      // [ ] Support network I/O trend analysis and forecasting
      // [ ] Implement network interface health monitoring (link status, errors, collisions)
      // [ ] Add network protocol breakdown (TCP, UDP, ICMP, IPv4/IPv6)
      // [ ] Support network usage attribution by process and connection
      // [ ] API/data structures defined & stable
      // [ ] Error handling + validation aligned with error taxonomy
      // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
      // [ ] Integration tests for external systems/contracts
      // [ ] Documentation: public API + system behavior
      // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
      // [ ] Security posture reviewed (inputs, authz, sandboxing)
      // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
      // [ ] Configurability and feature flags defined if relevant
      // [ ] Failure-mode cards documented (degradation paths)
      //
      // ACCEPTANCE CRITERIA:
      // - Network statistics use actual system APIs, not mock data
      // - All active network interfaces are monitored with accurate metrics
      // - Network latency measurements are within 1ms accuracy
      // - Packet loss detection works across different network conditions
      // - Network utilization alerting triggers at appropriate thresholds
      // - Trend analysis identifies network usage patterns and anomalies
      // - Interface health monitoring detects connectivity and performance issues
      // - Protocol breakdown provides detailed traffic composition analysis
      // - Process attribution shows network usage by application components
      // - Cross-platform compatibility works on Windows, Linux, and macOS
      // - Integration tests validate network monitoring against system utilities
      //
      // DEPENDENCIES:
      // - Cross-platform network monitoring libraries (Required)
      // - System network API access permissions (Required)
      // - Network latency measurement capabilities (Required)
      // - Statistical analysis for trending and forecasting (Required)
      // - Alerting and notification system (Required)
      //
      // ESTIMATED EFFORT: 16-20 hours (medium confidence)
      // PRIORITY: High
      // BLOCKING: No
      //
      // GOVERNANCE:
      // - CAWS Tier: 1 (infrastructure monitoring)
      // - Change Budget: ~350 LOC
      // - Reviewer Requirements: Network systems and infrastructure monitoring expertise
      // For now, return mock values

      // Mock network activity based on system load
      const loadAvg = loadavg()[0];
      const baseActivity = loadAvg * 1000; // Base activity in bytes/sec

      return {
        bytesInPerSecond: Math.round(baseActivity),
        bytesOutPerSecond: Math.round(baseActivity * 0.8),
      };
    } catch (error) {
      console.warn("Failed to collect network metrics:", error);
      return { bytesInPerSecond: 1000, bytesOutPerSecond: 800 }; // Neutral fallback
    }
  }

  /**
   * Get historical metrics for trend analysis
   */
  async getHistoricalMetrics(
    count: number = 10,
    startTime?: Date,
    endTime?: Date
  ): Promise<SystemMetrics[]> {
    if (!this.databaseClient) {
      console.warn("Database client not available for historical metrics");
      return [];
    }

    try {
      // Query system health metrics from database
      const pool = this.databaseClient["poolManager"].getPool();
      const client = await pool.connect();

      try {
        let query = `
          SELECT
            timestamp, cpu_usage, memory_usage, active_connections,
            queue_depth, error_rate, response_time_ms,
            created_at
          FROM system_health_metrics
        `;
        const params: any[] = [];
        const conditions: string[] = [];

        if (startTime) {
          conditions.push(`timestamp >= $${params.length + 1}`);
          params.push(startTime.toISOString());
        }

        if (endTime) {
          conditions.push(`timestamp <= $${params.length + 1}`);
          params.push(endTime.toISOString());
        }

        if (conditions.length > 0) {
          query += ` WHERE ${conditions.join(" AND ")}`;
        }

        query += ` ORDER BY timestamp DESC LIMIT $${params.length + 1}`;
        params.push(count);

        const result = await client.query(query, params);

        return result.rows.map((row) => ({
          cpuUsage: parseFloat(row.cpu_usage) || 0,
          memoryUsage: parseFloat(row.memory_usage) || 0,
          availableMemoryMB: 0, // Not stored in current schema
          totalMemoryMB: 0, // Not stored in current schema
          diskUsage: 0, // Not stored in current schema
          availableDiskGB: 0, // Not stored in current schema
          networkIO: { bytesInPerSecond: 0, bytesOutPerSecond: 0 }, // Not stored in current schema
          loadAverage: [0, 0, 0], // Not stored in current schema
          timestamp: new Date(row.timestamp),
        }));
      } finally {
        client.release();
      }
    } catch (error) {
      console.error("Failed to retrieve historical metrics:", error);
      return [];
    }
  }

  /**
   * Calculate metrics averages over a time period
   */
  calculateAverages(
    metrics: SystemMetrics[],
    _periodMs: number
  ): Partial<SystemMetrics> {
    if (metrics.length === 0) return {};

    const sum = metrics.reduce(
      (acc, m) => ({
        cpuUsage: acc.cpuUsage + m.cpuUsage,
        memoryUsage: acc.memoryUsage + m.memoryUsage,
        diskUsage: acc.diskUsage + m.diskUsage,
        networkIO: {
          bytesInPerSecond:
            acc.networkIO.bytesInPerSecond + m.networkIO.bytesInPerSecond,
          bytesOutPerSecond:
            acc.networkIO.bytesOutPerSecond + m.networkIO.bytesOutPerSecond,
        },
      }),
      {
        cpuUsage: 0,
        memoryUsage: 0,
        diskUsage: 0,
        networkIO: { bytesInPerSecond: 0, bytesOutPerSecond: 0 },
      }
    );

    return {
      cpuUsage: Math.round((sum.cpuUsage / metrics.length) * 100) / 100,
      memoryUsage: Math.round((sum.memoryUsage / metrics.length) * 100) / 100,
      diskUsage: Math.round((sum.diskUsage / metrics.length) * 100) / 100,
      networkIO: {
        bytesInPerSecond: Math.round(
          sum.networkIO.bytesInPerSecond / metrics.length
        ),
        bytesOutPerSecond: Math.round(
          sum.networkIO.bytesOutPerSecond / metrics.length
        ),
      },
    };
  }

  /**
   * Analyze trends in historical metrics
   */
  async analyzeTrends(
    hoursBack: number = 24,
    metricTypes: (keyof SystemMetrics)[] = ["cpuUsage", "memoryUsage"]
  ): Promise<{
    trends: Record<
      string,
      {
        direction: "increasing" | "decreasing" | "stable";
        changePercent: number;
        average: number;
        min: number;
        max: number;
        volatility: number;
      }
    >;
    analysisPeriod: { start: Date; end: Date };
  }> {
    const endTime = new Date();
    const startTime = new Date(endTime.getTime() - hoursBack * 60 * 60 * 1000);

    const historicalMetrics = await this.getHistoricalMetrics(
      1000,
      startTime,
      endTime
    );

    if (historicalMetrics.length < 2) {
      return {
        trends: {},
        analysisPeriod: { start: startTime, end: endTime },
      };
    }

    const trends: Record<string, any> = {};

    for (const metricType of metricTypes) {
      if (metricType === "timestamp") continue;

      const values = historicalMetrics
        .map((m) => {
          const value = (m as any)[metricType];
          if (typeof value === "number") return value;
          if (metricType === "networkIO") {
            return value?.bytesInPerSecond || 0;
          }
          if (metricType === "loadAverage") {
            return value?.[0] || 0;
          }
          return 0;
        })
        .filter((v) => v !== null && v !== undefined);

      if (values.length < 2) continue;

      const firstHalf = values.slice(0, Math.floor(values.length / 2));
      const secondHalf = values.slice(Math.floor(values.length / 2));

      const firstAvg =
        firstHalf.reduce((sum, v) => sum + v, 0) / firstHalf.length;
      const secondAvg =
        secondHalf.reduce((sum, v) => sum + v, 0) / secondHalf.length;

      const changePercent = ((secondAvg - firstAvg) / firstAvg) * 100;
      const overallAvg = values.reduce((sum, v) => sum + v, 0) / values.length;
      const min = Math.min(...values);
      const max = Math.max(...values);

      // Calculate volatility (standard deviation)
      const variance =
        values.reduce((sum, v) => sum + Math.pow(v - overallAvg, 2), 0) /
        values.length;
      const volatility = Math.sqrt(variance);

      let direction: "increasing" | "decreasing" | "stable" = "stable";
      if (Math.abs(changePercent) > 5) {
        // 5% threshold for significance
        direction = changePercent > 0 ? "increasing" : "decreasing";
      }

      trends[metricType] = {
        direction,
        changePercent: Math.round(changePercent * 100) / 100,
        average: Math.round(overallAvg * 100) / 100,
        min: Math.round(min * 100) / 100,
        max: Math.round(max * 100) / 100,
        volatility: Math.round(volatility * 100) / 100,
      };
    }

    return {
      trends,
      analysisPeriod: { start: startTime, end: endTime },
    };
  }

  /**
   * Get metrics summary with historical context
   */
  async getMetricsSummary(hoursBack: number = 24): Promise<{
    current: SystemMetrics | null;
    historical: {
      average: Partial<SystemMetrics>;
      trends: Record<string, any>;
      dataPoints: number;
    };
    alerts: string[];
  }> {
    const endTime = new Date();
    const startTime = new Date(endTime.getTime() - hoursBack * 60 * 60 * 1000);

    const historicalMetrics = await this.getHistoricalMetrics(
      1000,
      startTime,
      endTime
    );
    const current = historicalMetrics.length > 0 ? historicalMetrics[0] : null;
    const trends = await this.analyzeTrends(hoursBack);

    const alerts: string[] = [];

    // Generate alerts based on trends
    for (const [metric, trend] of Object.entries(trends.trends)) {
      if (
        trend.direction === "increasing" &&
        Math.abs(trend.changePercent) > 20
      ) {
        alerts.push(
          `${metric} is trending upward by ${trend.changePercent}% over ${hoursBack} hours`
        );
      }
      if (trend.volatility > 50) {
        alerts.push(`${metric} shows high volatility (${trend.volatility})`);
      }
    }

    return {
      current,
      historical: {
        average: this.calculateAverages(
          historicalMetrics,
          hoursBack * 60 * 60 * 1000
        ),
        trends: trends.trends,
        dataPoints: historicalMetrics.length,
      },
      alerts,
    };
  }
}
