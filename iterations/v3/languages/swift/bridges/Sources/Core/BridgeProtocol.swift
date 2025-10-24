// ============================================================================
// Core Bridge Protocol - Common Interface for All Bridges
// ============================================================================

import Foundation

// BridgeError is defined in ErrorHandling.swift

/// Result type for bridge operations
public typealias BridgeResult<T> = Result<T, BridgeError>

/// Common configuration for bridge operations
public struct BridgeConfig: Codable {
    public let timeoutSeconds: TimeInterval
    public let maxRetries: Int
    public let enableMetrics: Bool

    public init(
        timeoutSeconds: TimeInterval = 30.0,
        maxRetries: Int = 3,
        enableMetrics: Bool = true
    ) {
        self.timeoutSeconds = timeoutSeconds
        self.maxRetries = maxRetries
        self.enableMetrics = enableMetrics
    }
}

/// Protocol that all bridge implementations must conform to
public protocol BridgeProtocol {
    /// Bridge identifier
    var identifier: String { get }

    /// Bridge version
    var version: String { get }

    /// Bridge capabilities as a set of strings
    var capabilities: Set<String> { get }

    /// Initialize the bridge
    func initialize(config: BridgeConfig) -> BridgeResult<Void>

    /// Shutdown the bridge and cleanup resources
    func shutdown() -> BridgeResult<Void>

    /// Get bridge health status
    func healthCheck() -> BridgeResult<BridgeHealth>

    /// Get bridge metrics (if enabled)
    func getMetrics() -> BridgeResult<BridgeMetrics>
}

/// Bridge health status
public struct BridgeHealth: Codable {
    public let status: HealthStatus
    public let message: String?
    public let lastCheckTime: Date
    public let uptimeSeconds: TimeInterval

    public init(
        status: HealthStatus,
        message: String? = nil,
        lastCheckTime: Date = Date(),
        uptimeSeconds: TimeInterval = 0
    ) {
        self.status = status
        self.message = message
        self.lastCheckTime = lastCheckTime
        self.uptimeSeconds = uptimeSeconds
    }
}

/// Health status enumeration
public enum HealthStatus: String, Codable {
    case healthy
    case degraded
    case unhealthy
    case unknown
}

/// Bridge performance metrics
public struct BridgeMetrics: Codable {
    public let operationCount: Int64
    public let errorCount: Int64
    public let averageLatencyMs: Double
    public let p95LatencyMs: Double
    public let memoryUsageMB: Double
    public let lastUpdated: Date

    public init(
        operationCount: Int64 = 0,
        errorCount: Int64 = 0,
        averageLatencyMs: Double = 0,
        p95LatencyMs: Double = 0,
        memoryUsageMB: Double = 0,
        lastUpdated: Date = Date()
    ) {
        self.operationCount = operationCount
        self.errorCount = errorCount
        self.averageLatencyMs = averageLatencyMs
        self.p95LatencyMs = p95LatencyMs
        self.memoryUsageMB = memoryUsageMB
        self.lastUpdated = lastUpdated
    }
}

/// Thread-safe registry for managing bridge instances
public class BridgeRegistry {
    private let queue = DispatchQueue(label: "com.agent.bridge.registry", attributes: .concurrent)
    private var bridges: [String: any BridgeProtocol] = [:]

    /// Register a bridge instance
    public func register(_ bridge: any BridgeProtocol) {
        queue.async(flags: .barrier) {
            self.bridges[bridge.identifier] = bridge
        }
    }

    /// Unregister a bridge instance
    public func unregister(identifier: String) {
        queue.async(flags: .barrier) {
            self.bridges.removeValue(forKey: identifier)
        }
    }

    /// Get a bridge instance
    public func getBridge(identifier: String) -> (any BridgeProtocol)? {
        queue.sync {
            bridges[identifier]
        }
    }

    /// Get all registered bridge identifiers
    public func getAllBridgeIdentifiers() -> [String] {
        queue.sync {
            Array(bridges.keys)
        }
    }

    /// Get health status for all bridges
    public func getAllHealthStatuses() -> [String: BridgeHealth] {
        queue.sync {
            var statuses: [String: BridgeHealth] = [:]
            for (identifier, bridge) in bridges {
                if let health = try? bridge.healthCheck().get() {
                    statuses[identifier] = health
                } else {
                    statuses[identifier] = BridgeHealth(
                        status: .unhealthy,
                        message: "Health check failed",
                        uptimeSeconds: 0
                    )
                }
            }
            return statuses
        }
    }
}

/// Global bridge registry instance
public let globalBridgeRegistry = BridgeRegistry()
