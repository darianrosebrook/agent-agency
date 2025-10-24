// ============================================================================
// Text Tokenization Bridge
// ============================================================================

import Foundation
@_exported import Core

/// Text tokenization bridge conforming to BridgeProtocol
public class TokenizationBridge: BridgeProtocol {
    public let identifier = "TextTokenization"
    public let version = "1.0.0"
    public let capabilities: Set<String> = [
        "text_tokenization",
        "text_encoding",
        "text_decoding",
        "vocabulary_management"
    ]

    public init() {
        // Tokenization is ready to use immediately
    }

    public func initialize(config: BridgeConfig) -> BridgeResult<Void> {
        return .success(())
    }

    public func shutdown() -> BridgeResult<Void> {
        return .success(())
    }

    public func healthCheck() -> BridgeResult<BridgeHealth> {
        return .success(BridgeHealth(
            status: .healthy,
            message: "Tokenization operational",
            uptimeSeconds: 0
        ))
    }

    public func getMetrics() -> BridgeResult<BridgeMetrics> {
        return .success(BridgeMetrics())
    }

    // MARK: - Tokenization Operations

    /// Encode text to token IDs
    public func encode(text: String) throws -> [Int] {
        // Simple whitespace-based tokenization
        // In production, this would use actual model-specific tokenizers
        let words = text.split(separator: " ")
        return words.enumerated().map { (index, _) in
            1000 + index // Placeholder token IDs
        }
    }

    /// Decode token IDs to text
    public func decode(tokens: [Int]) throws -> String {
        // Simple reverse mapping
        let words = tokens.map { "word\($0)" }
        return words.joined(separator: " ")
    }

    /// Get vocabulary size
    public func getVocabSize() -> Int {
        return 32000 // Placeholder vocabulary size
    }

    /// Check if text fits within context window
    public func fitsInContext(text: String, maxLength: Int = 4096) -> Bool {
        do {
            let tokens = try encode(text: text)
            return tokens.count <= maxLength
        } catch {
            return false
        }
    }

    /// Truncate text to fit context window
    public func truncateToContext(text: String, maxLength: Int = 4096) -> String {
        do {
            let tokens = try encode(text: text)
            if tokens.count <= maxLength {
                return text
            }

            let truncatedTokens = Array(tokens.prefix(maxLength))
            return try decode(tokens: truncatedTokens)
        } catch {
            return text
        }
    }
}

// MARK: - Global Bridge Registration

private let _registration: Void = {
    globalBridgeRegistry.register(TokenizationBridge())
    return ()
}()
