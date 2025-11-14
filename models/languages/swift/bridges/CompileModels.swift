#!/usr/bin/env swift

import Foundation
import CoreML

class ModelCompiler {
    
    private let modelsDirectory: URL
    
    init() {
        // Get the models directory relative to this script
        let scriptURL = URL(fileURLWithPath: #file)
        let projectRoot = scriptURL.deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent()
        self.modelsDirectory = projectRoot.appendingPathComponent("coreml")
    }
    
    func compileAllModels() {
        print("🔨 Starting Core ML Model Compilation")
        print("📁 Models Directory: \(modelsDirectory.path)")
        print(String(repeating: "=", count: 60))
        
        let modelPaths = findModelPaths()
        
        for modelPath in modelPaths {
            compileModel(at: modelPath)
            print(String(repeating: "-", count: 60))
        }
        
        print("✅ Model compilation completed!")
    }
    
    private func findModelPaths() -> [URL] {
        var modelPaths: [URL] = []
        
        // Find all .mlmodel and .mlpackage files
        let fileManager = FileManager.default
        
        func searchDirectory(_ url: URL) {
            do {
                let contents = try fileManager.contentsOfDirectory(at: url, includingPropertiesForKeys: nil)
                
                for item in contents {
                    if item.pathExtension == "mlmodel" || item.pathExtension == "mlpackage" {
                        modelPaths.append(item)
                    } else if item.hasDirectoryPath {
                        searchDirectory(item)
                    }
                }
            } catch {
                print("⚠️  Error searching directory \(url.path): \(error)")
            }
        }
        
        searchDirectory(modelsDirectory)
        return modelPaths
    }
    
    private func compileModel(at url: URL) {
        let modelName = url.lastPathComponent
        let compiledURL = url.appendingPathExtension("mlmodelc")
        
        // Check if already compiled
        if FileManager.default.fileExists(atPath: compiledURL.path) {
            print("✅ \(modelName) - Already compiled")
            return
        }
        
        print("🔨 Compiling: \(modelName)")
        
        do {
            // Compile the model
            let compiledModelURL = try MLModel.compileModel(at: url)
            
            // Move to expected location if needed
            if compiledModelURL != compiledURL {
                try FileManager.default.moveItem(at: compiledModelURL, to: compiledURL)
            }
            
            print("✅ \(modelName) - Compiled successfully")
            
        } catch {
            print("❌ \(modelName) - Compilation failed: \(error)")
        }
    }
}

// Run the compiler
let compiler = ModelCompiler()
compiler.compileAllModels()
