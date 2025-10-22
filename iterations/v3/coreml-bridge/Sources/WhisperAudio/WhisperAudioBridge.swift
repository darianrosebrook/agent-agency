//
//  WhisperAudioBridge.swift
//  CoreMLBridge
//
//  Audio preprocessing bridge for Whisper model inference.
//  Handles audio resampling, normalization, and mel spectrogram generation.

import Foundation
import Accelerate
import AVFoundation

/// Audio preprocessing bridge for Whisper
@objc public class WhisperAudioBridge: NSObject {

    /// Audio preprocessing configuration
    public struct AudioConfig {
        public let sampleRate: Double = 16000.0
        public let nMels: Int = 80
        public let nFFT: Int = 400
        public let hopLength: Int = 160
        public let maxAudioLength: Double = 30.0

        public init() {}
    }

    private let config: AudioConfig

    public override init() {
        self.config = AudioConfig()
        super.init()
    }

    /// Preprocess audio file for Whisper inference
    /// - Parameters:
    ///   - audioPath: Path to audio file
    ///   - targetSampleRate: Target sample rate (default 16000)
    /// - Returns: MLMultiArray containing mel spectrogram
    @objc public func preprocessAudioFile(
        audioPath: String,
        targetSampleRate: Double = 16000.0
    ) throws -> MLMultiArray {
        // Load audio file
        let audioURL = URL(fileURLWithPath: audioPath)
        let audioFile = try AVAudioFile(forReading: audioURL)

        // Read audio data
        let audioBuffer = try readAudioBuffer(from: audioFile)

        // Convert to float array
        let audioSamples = try convertToFloatArray(audioBuffer)

        // Preprocess for Whisper
        return try preprocessAudioSamples(audioSamples, sampleRate: Double(audioFile.fileFormat.sampleRate))
    }

    /// Preprocess raw audio samples for Whisper inference
    /// - Parameters:
    ///   - samples: Audio samples as Float array
    ///   - sampleRate: Input sample rate
    /// - Returns: MLMultiArray containing mel spectrogram
    @objc public func preprocessAudioSamples(
        _ samples: [Float],
        sampleRate: Double
    ) throws -> MLMultiArray {
        // Resample to 16kHz if needed
        let resampledSamples = try resampleAudio(samples, fromRate: sampleRate, toRate: config.sampleRate)

        // Normalize audio
        let normalizedSamples = normalizeAudio(resampledSamples)

        // Pad or truncate to 30 seconds
        let processedSamples = padOrTruncateAudio(normalizedSamples, maxLengthSeconds: config.maxAudioLength)

        // Generate mel spectrogram
        let melSpectrogram = try generateMelSpectrogram(processedSamples)

        // Convert to MLMultiArray
        return try createMLMultiArray(from: melSpectrogram, shape: [NSNumber(value: config.nMels), NSNumber(value: melSpectrogram[0].count)])
    }

    /// Read audio buffer from AVAudioFile
    private func readAudioBuffer(from audioFile: AVAudioFile) throws -> AVAudioPCMBuffer {
        let frameCount = UInt32(audioFile.length)
        let audioBuffer = AVAudioPCMBuffer(pcmFormat: audioFile.processingFormat, frameCapacity: frameCount)!

        try audioFile.read(into: audioBuffer, frameCount: frameCount)
        return audioBuffer
    }

    /// Convert AVAudioPCMBuffer to Float array
    private func convertToFloatArray(_ buffer: AVAudioPCMBuffer) throws -> [Float] {
        guard let floatChannelData = buffer.floatChannelData else {
            throw NSError(domain: "WhisperAudioBridge", code: -1, userInfo: [NSLocalizedDescriptionKey: "Unable to get float channel data"])
        }

        let channelCount = Int(buffer.format.channelCount)
        let frameLength = Int(buffer.frameLength)

        // Use first channel for mono, or average channels for multi-channel
        var samples = [Float](repeating: 0.0, count: frameLength)

        if channelCount == 1 {
            // Mono
            let channelData = floatChannelData[0]
            for i in 0..<frameLength {
                samples[i] = channelData[i]
            }
        } else {
            // Multi-channel - average channels
            for channel in 0..<channelCount {
                let channelData = floatChannelData[channel]
                for i in 0..<frameLength {
                    samples[i] += channelData[i]
                }
            }
            // Average
            for i in 0..<frameLength {
                samples[i] /= Float(channelCount)
            }
        }

        return samples
    }

    /// Resample audio to target sample rate using linear interpolation
    private func resampleAudio(_ samples: [Float], fromRate: Double, toRate: Double) throws -> [Float] {
        guard fromRate != toRate else { return samples }

        let ratio = toRate / fromRate
        let newLength = Int(Double(samples.count) * ratio)

        var resampled = [Float](repeating: 0.0, count: newLength)

        for i in 0..<newLength {
            let srcIndex = Double(i) / ratio
            let indexFloor = Int(srcIndex)
            let indexCeil = min(indexFloor + 1, samples.count - 1)

            let fraction = srcIndex - Double(indexFloor)
            resampled[i] = samples[indexFloor] * Float(1.0 - fraction) + samples[indexCeil] * Float(fraction)
        }

        return resampled
    }

    /// Normalize audio to [-1, 1] range
    private func normalizeAudio(_ samples: [Float]) -> [Float] {
        guard let maxAbs = samples.max(by: { abs($0) < abs($1) }) else { return samples }

        if maxAbs > 0 {
            return samples.map { $0 / maxAbs }
        }

        return samples
    }

    /// Pad or truncate audio to maximum length
    private func padOrTruncateAudio(_ samples: [Float], maxLengthSeconds: Double) -> [Float] {
        let maxLengthSamples = Int(maxLengthSeconds * config.sampleRate)

        if samples.count >= maxLengthSamples {
            // Truncate
            return Array(samples.prefix(maxLengthSamples))
        } else {
            // Pad with zeros
            var padded = samples
            padded.append(contentsOf: [Float](repeating: 0.0, count: maxLengthSamples - samples.count))
            return padded
        }
    }

    /// Generate mel spectrogram from audio samples
    private func generateMelSpectrogram(_ samples: [Float]) throws -> [[Float]] {
        let nFrames = (samples.count - config.nFFT) / config.hopLength + 1

        // Compute STFT using Accelerate framework
        var realParts = [Float](repeating: 0.0, count: config.nFFT / 2)
        var imaginaryParts = [Float](repeating: 0.0, count: config.nFFT / 2)

        var spectrogram = [[Float]](repeating: [Float](repeating: 0.0, count: nFrames), count: config.nFFT / 2)

        // Simplified STFT computation
        for frame in 0..<nFrames {
            let startSample = frame * config.hopLength
            let endSample = min(startSample + config.nFFT, samples.count)

            if endSample - startSample == config.nFFT {
                // Apply window function (Hann window)
                var windowedSamples = Array(samples[startSample..<endSample])
                applyHannWindow(&windowedSamples)

                // Compute FFT
                computeFFT(&windowedSamples, realParts: &realParts, imaginaryParts: &imaginaryParts)

                // Compute magnitude spectrum
                for i in 0..<config.nFFT/2 {
                    let magnitude = sqrt(realParts[i] * realParts[i] + imaginaryParts[i] * imaginaryParts[i])
                    spectrogram[i][frame] = magnitude
                }
            }
        }

        // Convert to mel scale
        let melSpectrogram = convertToMelScale(spectrogram)

        // Convert to log scale
        return melSpectrogram.map { frame in
            frame.map { max($0, 1e-10).log(Float(M_E)) }
        }
    }

    /// Apply Hann window to samples
    private func applyHannWindow(_ samples: inout [Float]) {
        let n = samples.count
        for i in 0..<n {
            let windowValue = 0.5 * (1.0 - cos(2.0 * .pi * Float(i) / Float(n - 1)))
            samples[i] *= windowValue
        }
    }

    /// Compute FFT using Accelerate framework
    private func computeFFT(_ samples: inout [Float], realParts: inout [Float], imaginaryParts: inout [Float]) {
        let log2n = Int(log2(Float(samples.count)))
        let fftSize = 1 << log2n

        // Create FFT setup
        let fft = vDSP.FFT(log2n: vDSP_Length(log2n),
                          radix: .radix2,
                          ofType: DSPSplitComplex.self)!

        // Prepare split complex format
        var realp = [Float](repeating: 0.0, count: fftSize / 2)
        var imagp = [Float](repeating: 0.0, count: fftSize / 2)

        // Copy input (pad if necessary)
        var input = samples
        if input.count < fftSize {
            input.append(contentsOf: [Float](repeating: 0.0, count: fftSize - input.count))
        }

        input.withUnsafeBufferPointer { inputPtr in
            realp.withUnsafeMutableBufferPointer { realPtr in
                imagp.withUnsafeMutableBufferPointer { imagPtr in
                    var splitComplex = DSPSplitComplex(realp: realPtr.baseAddress!,
                                                     imagp: imagPtr.baseAddress!)

                    // Perform FFT
                    fft.forward(input: inputPtr.baseAddress!,
                              splitComplexOutput: &splitComplex)
                }
            }
        }

        // Copy results
        realParts = realp
        imaginaryParts = imagp
    }

    /// Convert linear spectrogram to mel scale
    private func convertToMelScale(_ spectrogram: [[Float]]) -> [[Float]] {
        let nMels = config.nMels
        let nFreqs = spectrogram.count

        // Mel filterbank (simplified)
        var melFilters = [[Float]](repeating: [Float](repeating: 0.0, count: nFreqs), count: nMels)

        // Create triangular mel filters
        let minMel = hzToMel(0.0)
        let maxMel = hzToMel(config.sampleRate / 2.0)

        for m in 0..<nMels {
            let melCenter = minMel + (maxMel - minMel) * Float(m) / Float(nMels - 1)
            let hzCenter = melToHz(melCenter)

            let binCenter = Int(hzCenter * Float(nFreqs) / (config.sampleRate / 2.0))
            let binLeft = max(0, binCenter - 1)
            let binRight = min(nFreqs - 1, binCenter + 1)

            // Create triangular filter
            for f in binLeft...binRight {
                let distance = abs(f - binCenter)
                if distance == 0 {
                    melFilters[m][f] = 1.0
                } else if distance == 1 {
                    melFilters[m][f] = 0.5
                }
            }
        }

        // Apply mel filters
        var melSpectrogram = [[Float]](repeating: [Float](repeating: 0.0, count: spectrogram[0].count), count: nMels)

        for m in 0..<nMels {
            for t in 0..<spectrogram[0].count {
                var sum = Float(0.0)
                for f in 0..<nFreqs {
                    sum += spectrogram[f][t] * melFilters[m][f]
                }
                melSpectrogram[m][t] = sum
            }
        }

        return melSpectrogram
    }

    /// Convert Hz to mel scale
    private func hzToMel(_ hz: Float) -> Float {
        return 2595.0 * log10(1.0 + hz / 700.0)
    }

    /// Convert mel scale to Hz
    private func melToHz(_ mel: Float) -> Float {
        return 700.0 * (pow(10.0, mel / 2595.0) - 1.0)
    }

    /// Create MLMultiArray from 2D float array
    private func createMLMultiArray(from data: [[Float]], shape: [NSNumber]) throws -> MLMultiArray {
        let totalElements = data.flatMap { $0 }.count
        let multiArray = try MLMultiArray(shape: shape, dataType: .float32)

        // Flatten 2D array and copy to MLMultiArray
        let flatData = data.flatMap { $0 }
        for (index, value) in flatData.enumerated() {
            multiArray[index] = NSNumber(value: value)
        }

        return multiArray
    }
}

/// C interface for Rust interop
@_cdecl("whisper_audio_preprocess_file")
public func whisper_audio_preprocess_file(
    audioPath: UnsafePointer<CChar>,
    outMultiArray: UnsafeMutablePointer<UnsafeMutableRawPointer?>,
    outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
) -> Int32 {
    do {
        let bridge = WhisperAudioBridge()
        let audioPathString = String(cString: audioPath)
        let multiArray = try bridge.preprocessAudioFile(audioPath: audioPathString)

        // Convert to raw pointer for Rust
        let retainedArray = Unmanaged.passRetained(multiArray).toOpaque()
        outMultiArray.pointee = retainedArray

        return 0 // Success
    } catch {
        let errorMessage = "Audio preprocessing failed: \(error.localizedDescription)"
        let cString = strdup(errorMessage)
        outError.pointee = cString
        return -1 // Error
    }
}

@_cdecl("whisper_audio_preprocess_samples")
public func whisper_audio_preprocess_samples(
    samples: UnsafePointer<Float>,
    sampleCount: Int,
    sampleRate: Double,
    outMultiArray: UnsafeMutablePointer<UnsafeMutableRawPointer?>,
    outError: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>
) -> Int32 {
    do {
        let bridge = WhisperAudioBridge()

        // Convert C array to Swift array
        let samplesArray = Array(UnsafeBufferPointer(start: samples, count: sampleCount))

        let multiArray = try bridge.preprocessAudioSamples(samplesArray, sampleRate: sampleRate)

        // Convert to raw pointer for Rust
        let retainedArray = Unmanaged.passRetained(multiArray).toOpaque()
        outMultiArray.pointee = retainedArray

        return 0 // Success
    } catch {
        let errorMessage = "Audio preprocessing failed: \(error.localizedDescription)"
        let cString = strdup(errorMessage)
        outError.pointee = cString
        return -1 // Error
    }
}

@_cdecl("whisper_audio_free_multiarray")
public func whisper_audio_free_multiarray(multiArrayPtr: UnsafeMutableRawPointer) {
    Unmanaged<MLMultiArray>.fromOpaque(multiArrayPtr).release()
}
