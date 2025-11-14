#!/usr/bin/env swift

import Foundation
import CoreML

let decoderURL = URL(fileURLWithPath: "/Users/darianrosebrook/Desktop/Projects/agent-agency/models/coreml/whisper/decoder.mlmodel")

do {
    let compiledURL = try MLModel.compileModel(at: decoderURL)
    print("✅ Decoder compiled successfully to: \(compiledURL.path)")
} catch {
    print("❌ Decoder compilation failed: \(error)")
}
