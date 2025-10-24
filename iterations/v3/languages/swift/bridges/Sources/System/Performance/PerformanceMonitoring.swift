// ============================================================================
// Performance Monitoring
// ============================================================================

import Foundation

/// Performance monitoring utilities
public class PerformanceMonitoring {

    private var startTimes: [String: Date] = [:]
    private var metrics: [String: PerformanceMetrics] = [:]

    /// Start timing an operation
    public func startTiming(_ operationId: String) {
        startTimes[operationId] = Date()
    }

    /// End timing an operation
    public func endTiming(_ operationId: String) -> TimeInterval? {
        guard let startTime = startTimes[operationId] else { return nil }
        let duration = Date().timeIntervalSince(startTime)
        startTimes.removeValue(forKey: operationId)

        // Record metrics
        var operationMetrics = metrics[operationId, default: PerformanceMetrics()]
        operationMetrics.samples.append(duration)
        metrics[operationId] = operationMetrics

        return duration
    }

    /// Get metrics for an operation
    public func getMetrics(for operationId: String) -> PerformanceMetrics? {
        return metrics[operationId]
    }

    /// Get all metrics
    public func getAllMetrics() -> [String: PerformanceMetrics] {
        return metrics
    }

    /// Reset all metrics
    public func reset() {
        startTimes.removeAll()
        metrics.removeAll()
    }
}

/// Performance metrics for an operation
public struct PerformanceMetrics {
    public var samples: [TimeInterval] = []
    public var minTime: TimeInterval { samples.min() ?? 0 }
    public var maxTime: TimeInterval { samples.max() ?? 0 }
    public var averageTime: TimeInterval { samples.reduce(0, +) / Double(samples.count) }
    public var count: Int { samples.count }
}
