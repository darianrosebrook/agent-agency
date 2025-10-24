// ============================================================================
// Whisper Bridge - Speech-to-Text Transcription
// ============================================================================

import Foundation
import CoreML
import Accelerate
import AVFoundation
import WhisperKit

/// Whisper speech-to-text bridge conforming to BridgeProtocol
public class WhisperBridge: BridgeProtocol {
    public let identifier = "WhisperSTT"
    public let version = "1.0.0"
    public let capabilities: Set<String> = [
        "speech_to_text",
        "audio_transcription",
        "timestamp_generation",
        "language_detection",
        "multilingual_support"
    ]

    private var whisperKit: WhisperKit?
    private var modelSize: String?
    private let queue = DispatchQueue(label: "com.agent.whisper", attributes: .concurrent)

    public init() {
        // Initialize without model - lazy loading
    }

    public func initialize(config: BridgeConfig) -> BridgeResult<Void> {
        // Model loading happens on first transcription request
        return .success(())
    }

    public func shutdown() -> BridgeResult<Void> {
        queue.sync(flags: .barrier) {
            self.whisperKit = nil
            self.modelSize = nil
        }
        return .success(())
    }

    public func healthCheck() -> BridgeResult<BridgeHealth> {
        return queue.sync {
            let isHealthy = whisperKit != nil
            return .success(BridgeHealth(
                status: isHealthy ? .healthy : .degraded,
                message: isHealthy ? "Whisper model loaded (\(modelSize ?? "unknown"))" : "Model not loaded",
                uptimeSeconds: 0
            ))
        }
    }

    public func getMetrics() -> BridgeResult<BridgeMetrics> {
        // Basic metrics - could be expanded with actual transcription stats
        return .success(BridgeMetrics())
    }

    // MARK: - Transcription Operations

    /// Transcribe audio file to text
    public func transcribe(
        audioPath: String,
        language: String? = nil,
        options: TranscriptionOptions = TranscriptionOptions()
    ) throws -> TranscriptionResult {
        try ensureModelLoaded()

        return try queue.sync {
            let audioURL = URL(fileURLWithPath: audioPath)

            // Create transcription request
            let transcriptionRequest = DecodingRequest(
                audioFile: audioURL,
                decodeOptions: DecodingOptions(
                    language: language,
                    task: options.task,
                    temperature: options.temperature,
                    temperatureInc: options.temperatureIncrement,
                    topK: options.topK,
                    topP: options.topP,
                    sampleLen: options.sampleLength,
                    bestOf: options.bestOf,
                    beamSize: options.beamSize,
                    patience: options.patience,
                    lengthPenalty: options.lengthPenalty,
                    repetitionPenalty: options.repetitionPenalty,
                    noRepeatNgramSize: options.noRepeatNgramSize,
                    compressionRatioThreshold: options.compressionRatioThreshold,
                    logprobThreshold: options.logprobThreshold,
                    noSpeechThreshold: options.noSpeechThreshold,
                    suppressTokens: options.suppressTokens,
                    initialPrompt: options.initialPrompt,
                    prefix: options.prefix,
                    suffix: options.suffix,
                    clipTimestamps: options.clipTimestamps,
                    hallucinationSilenceThresholds: options.hallucinationSilenceThresholds
                ),
                callback: nil
            )

            // Perform transcription
            let transcriptionResult = try self.whisperKit!.transcribe(audioPath: audioPath, decodeOptions: transcriptionRequest.decodeOptions)

            // Convert to our result format
            return TranscriptionResult(
                text: transcriptionResult.text,
                segments: transcriptionResult.segments.map { segment in
                    TranscriptionSegment(
                        text: segment.text,
                        startTime: segment.start,
                        endTime: segment.end,
                        confidence: segment.confidence
                    )
                },
                language: transcriptionResult.language,
                confidence: transcriptionResult.confidence ?? 0.0
            )
        }
    }

    /// Transcribe audio data directly
    public func transcribeAudioData(
        _ audioData: Data,
        language: String? = nil,
        options: TranscriptionOptions = TranscriptionOptions()
    ) throws -> TranscriptionResult {
        // Create temporary file for WhisperKit
        let tempURL = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString + ".wav")
        try audioData.write(to: tempURL)
        defer { try? FileManager.default.removeItem(at: tempURL) }

        return try transcribe(audioPath: tempURL.path, language: language, options: options)
    }

    /// Get supported languages
    public func getSupportedLanguages() -> [String] {
        return WhisperKit.supportedLanguages
    }

    /// Check if language is supported
    public func isLanguageSupported(_ language: String) -> Bool {
        return WhisperKit.supportedLanguages.contains(language)
    }

    // MARK: - Private Implementation

    private func ensureModelLoaded() throws {
        if whisperKit != nil { return }

        // Try to load model from cache first
        if let asset = try globalModelManager?.getCachedModel(identifier: "whisper-base", channel: .stable) {
            try loadModel(from: asset.localURL, modelSize: "base")
        } else {
            // Download model if not cached
            let asset = try globalModelManager!.downloadModel(identifier: "whisper-base", channel: .stable)
            try loadModel(from: asset.localURL, modelSize: "base")
        }
    }

    private func loadModel(from url: URL, modelSize: String) throws {
        // Initialize WhisperKit with model
        let config = WhisperKitConfig(model: modelSize)
        whisperKit = try WhisperKit(config)
        self.modelSize = modelSize
    }
}

// MARK: - Supporting Types

/// Transcription options
public struct TranscriptionOptions {
    public let task: DecodingTask
    public let temperature: [Float]
    public let temperatureIncrement: Float
    public let topK: Int
    public let topP: Float
    public let sampleLength: Int
    public let bestOf: Int
    public let beamSize: Int
    public let patience: Float
    public let lengthPenalty: Float
    public let repetitionPenalty: Float
    public let noRepeatNgramSize: Int
    public let compressionRatioThreshold: Float?
    public let logprobThreshold: Float
    public let noSpeechThreshold: Float
    public let suppressTokens: [Int]?
    public let initialPrompt: String?
    public let prefix: String?
    public let suffix: String?
    public let clipTimestamps: [Float]?
    public let hallucinationSilenceThresholds: [Float]?

    public init(
        task: DecodingTask = .transcribe,
        temperature: [Float] = [0.0, 0.2, 0.4, 0.6, 0.8, 1.0],
        temperatureIncrement: Float = 0.2,
        topK: Int = 5,
        topP: Float = 1.0,
        sampleLength: Int = 224,
        bestOf: Int = 5,
        beamSize: Int = 5,
        patience: Float = 1.0,
        lengthPenalty: Float = 1.0,
        repetitionPenalty: Float = 1.0,
        noRepeatNgramSize: Int = 0,
        compressionRatioThreshold: Float? = 2.4,
        logprobThreshold: Float = -1.0,
        noSpeechThreshold: Float = 0.6,
        suppressTokens: [Int]? = nil,
        initialPrompt: String? = nil,
        prefix: String? = nil,
        suffix: String? = nil,
        clipTimestamps: [Float]? = nil,
        hallucinationSilenceThresholds: [Float]? = nil
    ) {
        self.task = task
        self.temperature = temperature
        self.temperatureIncrement = temperatureIncrement
        self.topK = topK
        self.topP = topP
        self.sampleLength = sampleLength
        self.bestOf = bestOf
        self.beamSize = beamSize
        self.patience = patience
        self.lengthPenalty = lengthPenalty
        self.repetitionPenalty = repetitionPenalty
        self.noRepeatNgramSize = noRepeatNgramSize
        self.compressionRatioThreshold = compressionRatioThreshold
        self.logprobThreshold = logprobThreshold
        self.noSpeechThreshold = noSpeechThreshold
        self.suppressTokens = suppressTokens
        self.initialPrompt = initialPrompt
        self.prefix = prefix
        self.suffix = suffix
        self.clipTimestamps = clipTimestamps
        self.hallucinationSilenceThresholds = hallucinationSilenceThresholds
    }
}

/// Transcription result
public struct TranscriptionResult {
    public let text: String
    public let segments: [TranscriptionSegment]
    public let language: String?
    public let confidence: Float
}

/// Individual transcription segment with timing
public struct TranscriptionSegment {
    public let text: String
    public let startTime: Float
    public let endTime: Float
    public let confidence: Float
}

// MARK: - Global Bridge Registration

// Register this bridge globally
private let _registration: Void = {
    globalBridgeRegistry.register(WhisperBridge())
    return ()
}()

// MARK: - Global Model Manager Access

private var globalModelManager: ModelManager?

private func getModelManager() throws -> ModelManager {
    if let manager = globalModelManager {
        return manager
    }

    let manager = try ModelManager()
    globalModelManager = manager
    return manager
}
