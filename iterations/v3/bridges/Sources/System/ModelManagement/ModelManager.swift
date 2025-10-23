// ============================================================================
// Model Management - Content Addressing & Asset Governance
// ============================================================================

import Foundation
import CryptoKit
@_exported import Core

/// Model manager for content-addressed asset storage and lifecycle management
public class ModelManager {

    private let fileManager = FileManager.default
    private let queue = DispatchQueue(label: "com.agent.modelmanager", attributes: .concurrent)
    private var cache: [String: ModelMetadata] = [:]

    // MARK: - Configuration

    public struct Configuration {
        public let cacheDirectory: URL
        public let maxCacheSizeGB: Double
        public let enableContentVerification: Bool
        public let defaultChannel: ModelChannel

        public init(
            cacheDirectory: URL = ModelManager.defaultCacheDirectory(),
            maxCacheSizeGB: Double = 10.0,
            enableContentVerification: Bool = true,
            defaultChannel: ModelChannel = .stable
        ) {
            self.cacheDirectory = cacheDirectory
            self.maxCacheSizeGB = maxCacheSizeGB
            self.enableContentVerification = enableContentVerification
            self.defaultChannel = defaultChannel
        }
    }

    private let config: Configuration

    // MARK: - Initialization

    public init(configuration: Configuration = Configuration()) throws {
        self.config = configuration

        // Create cache directory if needed
        try fileManager.createDirectory(at: config.cacheDirectory, withIntermediateDirectories: true)

        // Load existing cache metadata
        try loadCacheMetadata()

        // Start background maintenance
        startMaintenanceTimer()
    }

    deinit {
        maintenanceTimer?.cancel()
    }

    // MARK: - Public API

    /// Download and cache a model by identifier and channel
    public func downloadModel(
        identifier: String,
        channel: ModelChannel = .stable,
        progressHandler: ((Double) -> Void)? = nil
    ) async throws -> ModelAsset {
        return try await withCheckedThrowingContinuation { continuation in
            queue.async {
                do {
                    let asset = try self.downloadModelSync(identifier: identifier, channel: channel, progressHandler: progressHandler)
                    continuation.resume(returning: asset)
                } catch {
                    continuation.resume(throwing: error)
                }
            }
        }
    }

    /// Get cached model asset
    public func getCachedModel(identifier: String, channel: ModelChannel = .stable) throws -> ModelAsset? {
        return try queue.sync {
            let key = cacheKey(identifier: identifier, channel: channel)
            guard let metadata = cache[key] else { return nil }

            // Verify asset still exists
            guard fileManager.fileExists(atPath: metadata.localPath.path) else {
                // Remove stale entry
                cache.removeValue(forKey: key)
                try saveCacheMetadata()
                return nil
            }

            return ModelAsset(metadata: metadata)
        }
    }

    /// List all cached models
    public func listCachedModels() -> [ModelAsset] {
        return queue.sync {
            cache.values.map { ModelAsset(metadata: $0) }
        }
    }

    /// Remove cached model
    public func removeCachedModel(identifier: String, channel: ModelChannel = .stable) throws {
        try queue.sync(flags: .barrier) {
            let key = cacheKey(identifier: identifier, channel: channel)
            guard let metadata = cache[key] else { return }

            // Remove file
            try fileManager.removeItem(at: metadata.localPath)

            // Remove from cache
            cache.removeValue(forKey: key)

            // Save updated cache
            try saveCacheMetadata()
        }
    }

    /// Get cache statistics
    public func getCacheStats() -> CacheStats {
        return queue.sync {
            let totalSize = cache.values.reduce(0) { $0 + $1.fileSize }
            let modelCount = cache.count
            let channels = Dictionary(grouping: cache.values) { $0.channel }
                .mapValues { $0.count }

            return CacheStats(
                totalSizeBytes: totalSize,
                modelCount: modelCount,
                channelCounts: channels,
                maxSizeBytes: Int64(config.maxCacheSizeGB * 1024 * 1024 * 1024)
            )
        }
    }

    /// Clear entire cache
    public func clearCache() throws {
        try queue.sync(flags: .barrier) {
            for metadata in cache.values {
                try fileManager.removeItem(at: metadata.localPath)
            }
            cache.removeAll()
            try saveCacheMetadata()
        }
    }

    // MARK: - Private Implementation

    private func downloadModelSync(
        identifier: String,
        channel: ModelChannel,
        progressHandler: ((Double) -> Void)?
    ) throws -> ModelAsset {
        let key = cacheKey(identifier: identifier, channel: channel)

        // Check if already cached
        if let metadata = cache[key],
           fileManager.fileExists(atPath: metadata.localPath.path) {
            // Verify content if enabled
            if config.enableContentVerification {
                try verifyContent(metadata: metadata)
            }
            return ModelAsset(metadata: metadata)
        }

        // Get download URL (placeholder - in real implementation would query registry)
        let downloadURL = try resolveDownloadURL(identifier: identifier, channel: channel)

        // Create local path
        let localPath = config.cacheDirectory.appendingPathComponent(key).appendingPathExtension("mlmodelc")

        // Download with progress
        try downloadFile(from: downloadURL, to: localPath, progressHandler: progressHandler)

        // Calculate hash and verify
        let hash = try calculateFileHash(at: localPath)
        let fileSize = try getFileSize(at: localPath)

        // Create metadata
        let metadata = ModelMetadata(
            identifier: identifier,
            channel: channel,
            version: "1.0.0", // Would come from registry
            hash: hash,
            fileSize: fileSize,
            localPath: localPath,
            downloadDate: Date(),
            lastAccessDate: Date()
        )

        // Verify content if enabled
        if config.enableContentVerification {
            try verifyContent(metadata: metadata)
        }

        // Cache metadata
        cache[key] = metadata
        try saveCacheMetadata()

        return ModelAsset(metadata: metadata)
    }

    private func resolveDownloadURL(identifier: String, channel: ModelChannel) throws -> URL {
        // Placeholder implementation - would query a model registry service
        // In production, this would make HTTP requests to a registry API

        let baseURL = "https://models.agent.agency"
        let channelPath = channel.rawValue
        let modelPath = "\(identifier).mlmodelc"

        guard let url = URL(string: "\(baseURL)/\(channelPath)/\(modelPath)") else {
            throw ModelError.invalidURL
        }

        return url
    }

    private func downloadFile(from url: URL, to localPath: URL, progressHandler: ((Double) -> Void)?) throws {
        let session = URLSession(configuration: .default)

        let semaphore = DispatchSemaphore(value: 0)
        var downloadError: Error?

        let task = session.downloadTask(with: url) { tempURL, response, error in
            defer { semaphore.signal() }

            if let error = error {
                downloadError = error
                return
            }

            guard let tempURL = tempURL else {
                downloadError = ModelError.downloadFailed("No temporary file")
                return
            }

            guard let httpResponse = response as? HTTPURLResponse,
                  (200...299).contains(httpResponse.statusCode) else {
                downloadError = ModelError.downloadFailed("HTTP error")
                return
            }

            do {
                // Move temp file to final location
                try self.fileManager.moveItem(at: tempURL, to: localPath)
            } catch {
                downloadError = error
            }
        }

        // Add progress tracking if requested
        if let progressHandler = progressHandler {
            // In a real implementation, would track download progress
            progressHandler(0.5) // Placeholder
        }

        task.resume()
        semaphore.wait()

        if let error = downloadError {
            throw error
        }
    }

    private func calculateFileHash(at url: URL) throws -> String {
        let data = try Data(contentsOf: url)
        let hash = SHA256.hash(data: data)
        return hash.compactMap { String(format: "%02x", $0) }.joined()
    }

    private func verifyContent(metadata: ModelMetadata) throws {
        // Verify file exists
        guard fileManager.fileExists(atPath: metadata.localPath.path) else {
            throw ModelError.contentVerificationFailed("File missing")
        }

        // Verify hash
        let currentHash = try calculateFileHash(at: metadata.localPath)
        guard currentHash == metadata.hash else {
            throw ModelError.contentVerificationFailed("Hash mismatch")
        }

        // Could also verify signature here if models are signed
    }

    private func getFileSize(at url: URL) throws -> Int64 {
        let attributes = try fileManager.attributesOfItem(atPath: url.path)
        return attributes[.size] as? Int64 ?? 0
    }

    private func cacheKey(identifier: String, channel: ModelChannel) -> String {
        return "\(identifier)_\(channel.rawValue)"
    }

    private func loadCacheMetadata() throws {
        let metadataURL = config.cacheDirectory.appendingPathComponent("cache_metadata.json")

        guard fileManager.fileExists(atPath: metadataURL.path) else {
            return // No existing cache
        }

        let data = try Data(contentsOf: metadataURL)
        let decoder = JSONDecoder()
        decoder.dateDecodingStrategy = .iso8601

        let cachedMetadata = try decoder.decode([String: ModelMetadata].self, from: data)
        cache = cachedMetadata
    }

    private func saveCacheMetadata() throws {
        let metadataURL = config.cacheDirectory.appendingPathComponent("cache_metadata.json")

        let encoder = JSONEncoder()
        encoder.dateEncodingStrategy = .iso8601
        encoder.outputFormatting = .prettyPrinted

        let data = try encoder.encode(cache)
        try data.write(to: metadataURL)
    }

    // MARK: - Maintenance

    private var maintenanceTimer: DispatchSourceTimer?

    private func startMaintenanceTimer() {
        maintenanceTimer = DispatchSource.makeTimerSource(queue: queue)
        maintenanceTimer?.schedule(deadline: .now() + 3600, repeating: 3600) // Hourly

        maintenanceTimer?.setEventHandler {
            self.performMaintenance()
        }

        maintenanceTimer?.resume()
    }

    private func performMaintenance() {
        do {
            // Clean up stale entries
            let staleKeys = cache.filter { _, metadata in
                !fileManager.fileExists(atPath: metadata.localPath.path)
            }.keys

            for key in staleKeys {
                cache.removeValue(forKey: key)
            }

            // Enforce size limits
            try enforceCacheSizeLimit()

            // Save updated metadata
            try saveCacheMetadata()

        } catch {
            print("ModelManager maintenance failed: \(error)")
        }
    }

    private func enforceCacheSizeLimit() throws {
        let stats = getCacheStats()
        guard stats.totalSizeBytes > stats.maxSizeBytes else { return }

        // Sort by last access date (oldest first)
        let sortedEntries = cache.sorted { $0.value.lastAccessDate < $1.value.lastAccessDate }

        var currentSize = stats.totalSizeBytes
        var removedCount = 0

        for (key, metadata) in sortedEntries {
            guard currentSize > stats.maxSizeBytes else { break }

            try fileManager.removeItem(at: metadata.localPath)
            cache.removeValue(forKey: key)
            currentSize -= metadata.fileSize
            removedCount += 1
        }

        if removedCount > 0 {
            print("Removed \(removedCount) cached models to enforce size limit")
        }
    }

    // MARK: - Static Helpers

    public static func defaultCacheDirectory() -> URL {
        let fileManager = FileManager.default
        let appSupport = fileManager.urls(for: .applicationSupportDirectory, in: .userDomainMask).first!
        return appSupport.appendingPathComponent("AgentAgency").appendingPathComponent("Models")
    }
}

// MARK: - Supporting Types

/// Model channel for versioning and stability
public enum ModelChannel: String, Codable {
    case stable
    case canary
    case experimental
}

/// Model asset representing a cached model
public struct ModelAsset {
    public let metadata: ModelMetadata
    public var localURL: URL { metadata.localPath }

    public init(metadata: ModelMetadata) {
        self.metadata = metadata
    }
}

/// Model metadata for caching and verification
public struct ModelMetadata: Codable {
    public let identifier: String
    public let channel: ModelChannel
    public let version: String
    public let hash: String
    public let fileSize: Int64
    public let localPath: URL
    public let downloadDate: Date
    public var lastAccessDate: Date

    public init(
        identifier: String,
        channel: ModelChannel,
        version: String,
        hash: String,
        fileSize: Int64,
        localPath: URL,
        downloadDate: Date,
        lastAccessDate: Date
    ) {
        self.identifier = identifier
        self.channel = channel
        self.version = version
        self.hash = hash
        self.fileSize = fileSize
        self.localPath = localPath
        self.downloadDate = downloadDate
        self.lastAccessDate = lastAccessDate
    }
}

/// Cache statistics
public struct CacheStats {
    public let totalSizeBytes: Int64
    public let modelCount: Int
    public let channelCounts: [ModelChannel: Int]
    public let maxSizeBytes: Int64

    public var totalSizeGB: Double {
        Double(totalSizeBytes) / (1024 * 1024 * 1024)
    }

    public var utilizationPercent: Double {
        Double(totalSizeBytes) / Double(maxSizeBytes) * 100
    }
}

/// Model manager errors
public enum ModelError: Error {
    case invalidURL
    case downloadFailed(String)
    case contentVerificationFailed(String)
    case cacheCorrupted
    case diskSpaceInsufficient
}

// MARK: - Bridge Protocol Conformance

extension ModelManager: BridgeProtocol {
    public var identifier: String { "ModelManager" }
    public var version: String { "1.0.0" }
    public var capabilities: Set<String> { ["model_download", "model_cache", "content_verification"] }

    public func initialize(config: BridgeConfig) -> BridgeResult<Void> {
        // Already initialized in init
        return .success(())
    }

    public func shutdown() -> BridgeResult<Void> {
        maintenanceTimer?.cancel()
        return .success(())
    }

    public func healthCheck() -> BridgeResult<BridgeHealth> {
        do {
            // Check if cache directory is accessible
            let stats = getCacheStats()

            let status: HealthStatus = stats.totalSizeBytes < stats.maxSizeBytes ? .healthy : .degraded

            return .success(BridgeHealth(
                status: status,
                message: "Cache utilization: \(String(format: "%.1f", stats.utilizationPercent))%",
                uptimeSeconds: 0 // Would track actual uptime
            ))
        } catch {
            return .failure(.resourceUnavailable("Health check failed: \(error.localizedDescription)"))
        }
    }

    public func getMetrics() -> BridgeResult<BridgeMetrics> {
        let stats = getCacheStats()
        return .success(BridgeMetrics(
            operationCount: Int64(stats.modelCount),
            errorCount: 0, // Would track actual errors
            averageLatencyMs: 0, // Would track download times
            p95LatencyMs: 0,
            memoryUsageMB: stats.totalSizeGB * 1024,
            lastUpdated: Date()
        ))
    }
}
