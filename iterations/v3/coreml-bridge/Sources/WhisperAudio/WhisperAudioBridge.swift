import Foundation
import CoreML
import Accelerate
import AVFoundation

/// Simplified Whisper audio preprocessing bridge
/// This provides basic audio processing for testing

@_cdecl("whisper_audio_test")
public func whisper_audio_test() -> Int32 {
    // Test function that returns 24 (kHz for Whisper)
    return 16000
}

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

@_cdecl("whisper_audio_free_multiarray")
public func whisper_audio_free_multiarray(multiArrayPtr: UnsafeMutableRawPointer?) {
    guard let ptr = multiArrayPtr else { return }
    let _ = Unmanaged<MLMultiArray>.fromOpaque(ptr).takeRetainedValue()
}