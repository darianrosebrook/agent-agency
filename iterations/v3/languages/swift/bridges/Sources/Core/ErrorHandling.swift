// ============================================================================
// Error Handling - Consistent Error Management Across Bridges
// ============================================================================

import Foundation

/// Comprehensive error types for bridge operations
public enum BridgeError: LocalizedError {
    // Core errors
    case initializationFailed(String)
    case invalidInput(String)
    case resourceUnavailable(String)
    case timeout(String)

    // Model errors
    case modelLoadFailed(String)
    case modelNotFound(String)
    case modelIncompatible(String)
    case modelCorrupted(String)

    // Inference errors
    case inferenceFailed(String)
    case invalidModelOutput(String)
    case processingFailed(String)

    // Audio/Video errors
    case audioProcessingFailed(String)
    case videoProcessingFailed(String)
    case unsupportedFormat(String)

    // Text processing errors
    case tokenizationFailed(String)
    case encodingFailed(String)
    case decodingFailed(String)
    case sequenceTooLong(String)

    // System errors
    case memoryAllocationFailed(String)
    case diskSpaceInsufficient(String)
    case permissionDenied(String)

    // Network errors
    case networkUnavailable(String)
    case downloadFailed(String)

    public var errorDescription: String? {
        switch self {
        case .initializationFailed(let msg):
            return "Initialization failed: \(msg)"
        case .invalidInput(let msg):
            return "Invalid input: \(msg)"
        case .resourceUnavailable(let msg):
            return "Resource unavailable: \(msg)"
        case .timeout(let msg):
            return "Operation timeout: \(msg)"
        case .modelLoadFailed(let msg):
            return "Model load failed: \(msg)"
        case .modelNotFound(let msg):
            return "Model not found: \(msg)"
        case .modelIncompatible(let msg):
            return "Model incompatible: \(msg)"
        case .modelCorrupted(let msg):
            return "Model corrupted: \(msg)"
        case .inferenceFailed(let msg):
            return "Inference failed: \(msg)"
        case .invalidModelOutput(let msg):
            return "Invalid model output: \(msg)"
        case .processingFailed(let msg):
            return "Processing failed: \(msg)"
        case .audioProcessingFailed(let msg):
            return "Audio processing failed: \(msg)"
        case .videoProcessingFailed(let msg):
            return "Video processing failed: \(msg)"
        case .unsupportedFormat(let msg):
            return "Unsupported format: \(msg)"
        case .tokenizationFailed(let msg):
            return "Tokenization failed: \(msg)"
        case .encodingFailed(let msg):
            return "Encoding failed: \(msg)"
        case .decodingFailed(let msg):
            return "Decoding failed: \(msg)"
        case .sequenceTooLong(let msg):
            return "Sequence too long: \(msg)"
        case .memoryAllocationFailed(let msg):
            return "Memory allocation failed: \(msg)"
        case .diskSpaceInsufficient(let msg):
            return "Insufficient disk space: \(msg)"
        case .permissionDenied(let msg):
            return "Permission denied: \(msg)"
        case .networkUnavailable(let msg):
            return "Network unavailable: \(msg)"
        case .downloadFailed(let msg):
            return "Download failed: \(msg)"
        }
    }

    /// Error severity for logging and monitoring
    public var severity: ErrorSeverity {
        switch self {
        case .modelCorrupted, .memoryAllocationFailed, .diskSpaceInsufficient:
            return .critical
        case .modelLoadFailed, .inferenceFailed, .processingFailed:
            return .high
        case .modelNotFound, .modelIncompatible, .invalidInput:
            return .medium
        case .timeout, .resourceUnavailable, .networkUnavailable:
            return .low
        default:
            return .low
        }
    }

    /// Whether this error is retryable
    public var isRetryable: Bool {
        switch self {
        case .timeout, .resourceUnavailable, .networkUnavailable, .downloadFailed:
            return true
        default:
            return false
        }
    }
}

/// Error severity levels
public enum ErrorSeverity: String, Codable {
    case low
    case medium
    case high
    case critical
}

/// Error context for better debugging
public struct ErrorContext {
    public let operation: String
    public let timestamp: Date
    public let bridgeIdentifier: String
    public let additionalInfo: [String: String]

    public init(
        operation: String,
        bridgeIdentifier: String,
        additionalInfo: [String: String] = [:]
    ) {
        self.operation = operation
        self.timestamp = Date()
        self.bridgeIdentifier = bridgeIdentifier
        self.additionalInfo = additionalInfo
    }
}

/// Error logger for consistent error reporting
public class ErrorLogger {
    private let queue = DispatchQueue(label: "com.agent.error.logger")

    /// Log an error with context
    public func logError(_ error: BridgeError, context: ErrorContext) {
        queue.async {
            let logMessage = """
            [\(context.timestamp)] \(error.severity.rawValue.uppercased()) \
            [\(context.bridgeIdentifier)] \(context.operation): \
            \(error.localizedDescription)
            """

            if !context.additionalInfo.isEmpty {
                let infoString = context.additionalInfo.map { "\($0.key)=\($0.value)" }.joined(separator: ", ")
                print("\(logMessage) (\(infoString))")
            } else {
                print(logMessage)
            }

            // In production, this would integrate with system logging
            // NSLog, OSLog, or custom logging framework
        }
    }

    /// Log an error without context
    public func logError(_ error: BridgeError, operation: String, bridgeId: String) {
        let context = ErrorContext(operation: operation, bridgeIdentifier: bridgeId)
        logError(error, context: context)
    }
}

/// Global error logger instance
public let globalErrorLogger = ErrorLogger()

/// Extension for converting NSError to BridgeError
extension BridgeError {
    public static func fromNSError(_ error: NSError, operation: String) -> BridgeError {
        let message = error.localizedDescription

        // Map common NSError domains to BridgeError
        switch error.domain {
        case NSCocoaErrorDomain:
            return .processingFailed("Cocoa error in \(operation): \(message)")
        case NSURLErrorDomain:
            return .networkUnavailable("Network error in \(operation): \(message)")
        default:
            return .processingFailed("System error in \(operation): \(message)")
        }
    }
}

/// Utility for executing operations with error handling
public class ErrorHandling {

    /// Execute an operation with automatic error logging
    /// - Parameters:
    ///   - operation: Operation to execute
    ///   - operationName: Name of the operation for logging
    ///   - bridgeId: Bridge identifier for logging
    ///   - additionalInfo: Additional context for error logging
    /// - Returns: Operation result or nil on error
    public static func executeWithLogging<T>(
        operation: () throws -> T,
        operationName: String,
        bridgeId: String,
        additionalInfo: [String: String] = [:]
    ) -> T? {
        do {
            return try operation()
        } catch let error as BridgeError {
            let context = ErrorContext(
                operation: operationName,
                bridgeIdentifier: bridgeId,
                additionalInfo: additionalInfo
            )
            globalErrorLogger.logError(error, context: context)
            return nil
        } catch {
            let bridgeError = BridgeError.processingFailed("Unexpected error: \(error.localizedDescription)")
            let context = ErrorContext(
                operation: operationName,
                bridgeIdentifier: bridgeId,
                additionalInfo: additionalInfo
            )
            globalErrorLogger.logError(bridgeError, context: context)
            return nil
        }
    }

    /// Execute an operation with retry logic
    /// - Parameters:
    ///   - operation: Operation to execute (should be idempotent)
    ///   - maxRetries: Maximum number of retry attempts
    ///   - retryDelay: Delay between retries
    ///   - operationName: Name of the operation for logging
    ///   - bridgeId: Bridge identifier for logging
    /// - Returns: Operation result or nil on final failure
    public static func executeWithRetry<T>(
        operation: () throws -> T,
        maxRetries: Int = 3,
        retryDelay: TimeInterval = 1.0,
        operationName: String,
        bridgeId: String
    ) -> T? {
        var currentRetryDelay = retryDelay
        var lastError: Error?

        for attempt in 0...maxRetries {
            do {
                return try operation()
            } catch let error as BridgeError where error.isRetryable && attempt < maxRetries {
                lastError = error
                let context = ErrorContext(
                    operation: "\(operationName) (attempt \(attempt + 1)/\(maxRetries + 1))",
                    bridgeIdentifier: bridgeId,
                    additionalInfo: ["will_retry": "true", "delay_seconds": "\(retryDelay)"]
                )
                globalErrorLogger.logError(error, context: context)

                if attempt < maxRetries {
                    Thread.sleep(forTimeInterval: currentRetryDelay)
                    // Exponential backoff
                    currentRetryDelay *= 2
                }
            } catch let error as BridgeError {
                // Non-retryable error
                lastError = error
                let context = ErrorContext(
                    operation: operationName,
                    bridgeIdentifier: bridgeId,
                    additionalInfo: ["retryable": "false"]
                )
                globalErrorLogger.logError(error, context: context)
                break
            } catch {
                // Unexpected error, treat as non-retryable
                lastError = error
                let bridgeError = BridgeError.processingFailed("Unexpected error: \(error.localizedDescription)")
                let context = ErrorContext(
                    operation: operationName,
                    bridgeIdentifier: bridgeId,
                    additionalInfo: ["retryable": "false", "unexpected": "true"]
                )
                globalErrorLogger.logError(bridgeError, context: context)
                break
            }
        }

        // Log final failure
        if let error = lastError {
            let context = ErrorContext(
                operation: operationName,
                bridgeIdentifier: bridgeId,
                additionalInfo: ["final_failure": "true", "max_retries_exceeded": "true"]
            )
            globalErrorLogger.logError(
                BridgeError.processingFailed("Operation failed after \(maxRetries + 1) attempts: \(error.localizedDescription)"),
                context: context
            )
        }

        return nil
    }
}
