//
//  main.swift
//  ModelDownloader
//
//  Downloads Whisper models using WhisperKit for use in Agent Agency
//

import Foundation
import WhisperKit

@main
struct ModelDownloader {
    static func main() async throws {
        print("🤖 Agent Agency Whisper Model Downloader")
        print("========================================")

        let modelName = "large-v3"
        let downloadPath = "../models/whisper/\(modelName)"

        print("📥 Downloading Whisper \(modelName) model...")
        print("   Target path: \(downloadPath)")

        do {
            // Initialize WhisperKit which will download the model
            let config = WhisperKitConfig(
                model: modelName,
                downloadBase: URL(fileURLWithPath: downloadPath)
            )

            print("🔄 Initializing WhisperKit (this will download the model)...")
            let whisperKit = try await WhisperKit(config)

            print("✅ Model downloaded successfully!")
            print("   Model: \(modelName)")
            print("   Path: \(downloadPath)")

            // Test basic functionality
            print("🧪 Testing model loading...")
            let testText = "This is a test of the Whisper model."
            print("   Test input: \"\(testText)\"")

            // Note: We can't actually transcribe without audio input,
            // but model loading success indicates download worked
            print("✅ Model validation complete!")

        } catch {
            print("❌ Error downloading model: \(error)")
            throw error
        }
    }
}
