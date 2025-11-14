#!/usr/bin/env swift

import Foundation

let scriptURL = URL(fileURLWithPath: #file)
let projectRoot = scriptURL.deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent()
let modelsDirectory = projectRoot.appendingPathComponent("coreml")

print("Script URL: \(scriptURL.path)")
print("Project Root: \(projectRoot.path)")
print("Models Directory: \(modelsDirectory.path)")
print("Directory exists: \(FileManager.default.fileExists(atPath: modelsDirectory.path))")
