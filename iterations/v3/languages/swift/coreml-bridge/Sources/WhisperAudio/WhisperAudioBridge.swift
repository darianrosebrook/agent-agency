import Foundation
import CoreML
import Accelerate
import AVFoundation
import WhisperKit

/// WhisperKit integration for speech-to-text transcription
/// Provides complete audio preprocessing and model inference

// Global WhisperKit instance for reuse
private var whisperKitInstance: WhisperKit?

/// Initialize WhisperKit with specified model
@_cdecl("whisper_init_model")
public func whisper_init_model(
    modelPath: UnsafePointer<CChar>,
    modelSize: UnsafePointer<CChar>,
    outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
) -> Int32 {
    autoreleasepool {
        let _ = String(cString: modelPath)
        let size = String(cString: modelSize)

        print("Initializing WhisperKit with model: \(size)")

        // Use semaphore to block until async initialization completes
        let semaphore = DispatchSemaphore(value: 0)
        var initResult: Result<WhisperKit, Error>?

        Task {
            do {
                let config = WhisperKitConfig(model: size)
                let whisperKit = try await WhisperKit(config)
                initResult = .success(whisperKit)
            } catch {
                initResult = .failure(error)
            }
            semaphore.signal()
        }

        // Wait for initialization to complete
        semaphore.wait()

        guard let result = initResult else {
            let errorStr = strdup("WhisperKit initialization semaphore failed")
            outError.pointee = errorStr
            return 1
        }

        switch result {
        case .success(let whisperKit):
            // Store the instance globally for reuse
            whisperKitInstance = whisperKit
            print("WhisperKit initialized successfully")
            outError.pointee = nil
            return 0

        case .failure(let error):
            let errorStr = strdup("WhisperKit initialization failed: \(error.localizedDescription)")
            outError.pointee = errorStr
            return 1
        }
    }
}

/// Transcribe audio from file path (placeholder implementation with WhisperKit preparation)
@_cdecl("whisper_transcribe_file")
public func whisper_transcribe_file(
    audioPath: UnsafePointer<CChar>,
    language: UnsafePointer<CChar>?,
    outText: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>,
    outSegments: UnsafeMutablePointer<UnsafeMutableRawPointer?>,
    outConfidence: UnsafeMutablePointer<Float>,
    outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
) -> Int32 {
    autoreleasepool {
        let path = String(cString: audioPath)
        let lang = language.map { String(cString: $0) }

        print("WhisperKit transcribe: path=\(path), lang=\(lang ?? "auto")")

        // Ensure WhisperKit is initialized
        guard let whisperKit = whisperKitInstance else {
            let errorStr = strdup("WhisperKit not initialized. Call whisper_init_model first.")
            outError.pointee = errorStr
            return 1
        }

        // Use semaphore to block until async transcription completes
        let semaphore = DispatchSemaphore(value: 0)
        var transcriptionResult: Result<[TranscriptionResult], Error>?

        Task {
            do {
                let results = try await whisperKit.transcribe(audioPath: path)
                transcriptionResult = .success(results)
            } catch {
                transcriptionResult = .failure(error)
            }
            semaphore.signal()
        }

        // Wait for transcription to complete
        semaphore.wait()

        guard let result = transcriptionResult else {
            let errorStr = strdup("Transcription semaphore failed")
            outError.pointee = errorStr
            return 1
        }

        switch result {
        case .success(let transcriptionResults):
            // Use the first result (should typically be only one for single file)
            guard let transcriptionResult = transcriptionResults.first else {
                let errorStr = strdup("No transcription results returned")
                outError.pointee = errorStr
                return 1
            }

            // Extract text
            let text = transcriptionResult.text

            // Calculate confidence from segment average log probabilities
            let confidence: Float
            if !transcriptionResult.segments.isEmpty {
                let avgLogProbs = transcriptionResult.segments.map { $0.avgLogprob }
                confidence = Float(avgLogProbs.reduce(0, +) / Float(avgLogProbs.count))
            } else {
                confidence = 0.95 // Default confidence when no segments available
            }

            print("Transcription completed: confidence=\(confidence), text length=\(text.count)")

            // Convert segments to JSON
            do {
                let encoder = JSONEncoder()
                encoder.outputFormatting = .prettyPrinted
                let segmentsData = try encoder.encode(transcriptionResult.segments)
                let segmentsString = String(data: segmentsData, encoding: .utf8) ?? "[]"

                // Return results
                outText.pointee = strdup(text)
                let segmentsNSString = NSString(string: segmentsString)
                outSegments.pointee = Unmanaged.passRetained(segmentsNSString).toOpaque()
                outConfidence.pointee = confidence
                outError.pointee = nil
                return 0

            } catch {
                let errorStr = strdup("Failed to encode segments to JSON: \(error.localizedDescription)")
                outError.pointee = errorStr
                return 1
            }

        case .failure(let error):
            let errorStr = strdup("Transcription failed: \(error.localizedDescription)")
            outError.pointee = errorStr
            return 1
        }
    }
}

/// Audio preprocessing for Whisper (placeholder implementation)
@_cdecl("whisper_audio_preprocess_file")
public func whisper_audio_preprocess_file(
    audioPath: UnsafePointer<CChar>,
    outMultiArray: UnsafeMutablePointer<UnsafeMutableRawPointer?>,
    outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
) -> Int32 {
    // Simplified implementation - create a dummy multiarray for testing
    autoreleasepool {
        do {
            let shape = [1, 80, 3000] as [NSNumber]
            let multiArray = try MLMultiArray(shape: shape, dataType: .float32)

            // Fill with some test data
            for i in 0..<multiArray.count {
                multiArray[i] = NSNumber(value: Float.random(in: -1.0...1.0))
            }

            // Return the multiarray (will be managed by caller)
            let unmanaged = Unmanaged.passRetained(multiArray)
            outMultiArray.pointee = unmanaged.toOpaque()
            outError.pointee = nil
            return 0
        } catch {
            let errorStr = strdup(error.localizedDescription)
            outError.pointee = errorStr
            outMultiArray.pointee = nil
            return 1
        }
    }
}

/// Test function
@_cdecl("whisper_audio_test")
public func whisper_audio_test() -> Int32 {
    return 16000 // Whisper sample rate
}

@_cdecl("whisper_audio_free_multiarray")
public func whisper_audio_free_multiarray(multiArrayPtr: UnsafeMutableRawPointer?) {
    guard let ptr = multiArrayPtr else { return }
    let _ = Unmanaged<MLMultiArray>.fromOpaque(ptr).takeRetainedValue()
}