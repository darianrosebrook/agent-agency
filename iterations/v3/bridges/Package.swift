// swift-tools-version: 5.10
import PackageDescription

let package = Package(
    name: "AgentBridges",
    platforms: [.macOS(.v13), .iOS(.v16)],
    products: [
        .library(name: "AgentBridges", targets: ["AgentBridges"]),
        .library(name: "AudioBridges", targets: ["Audio.STT.Whisper", "Audio.STT.SpeechFramework", "Audio.Utils"]),
        .library(name: "VisionBridges", targets: ["Vision.OD.YOLO", "Vision.OCR.VisionOCR", "Vision.ImageUtils"]),
        .library(name: "TextBridges", targets: ["Text.LLM.Mistral", "Text.Tokenization", "Text.Generation.Diffusion"]),
        .library(name: "BridgesFFI", targets: ["BridgesFFI"]) // C ABI surface
    ],
    dependencies: [
        .package(url: "https://github.com/argmaxinc/whisperkit", from: "0.14.0")
    ],
    targets: [
        .target(name: "Core", path: "Sources/Core",
                swiftSettings: [.define("BRIDGES_CORE")]),

        .target(name: "Audio.Utils", dependencies: ["Core"], path: "Sources/Audio/AudioProcessing",
                swiftSettings: [.define("ENABLE_AUDIO_BRIDGES")]),
        .target(name: "Audio.STT.Whisper", dependencies: ["Core","Audio.Utils"],
                path: "Sources/Audio/SpeechToText/Whisper",
                swiftSettings: [.define("ENABLE_AUDIO_BRIDGES")]),
        .target(name: "Audio.STT.SpeechFramework", dependencies: ["Core","Audio.Utils"],
                path: "Sources/Audio/SpeechToText/SpeechFramework",
                swiftSettings: [.define("ENABLE_AUDIO_BRIDGES")]),

        .target(name: "Vision.ImageUtils", dependencies: ["Core"], path: "Sources/Vision/ImageProcessing",
                swiftSettings: [.define("ENABLE_VISION_BRIDGES")]),
        .target(name: "Vision.OD.YOLO", dependencies: ["Core","Vision.ImageUtils"],
                path: "Sources/Vision/ObjectDetection/YOLO",
                swiftSettings: [.define("ENABLE_VISION_BRIDGES")]),
        .target(name: "Vision.OCR.VisionOCR", dependencies: ["Core","Vision.ImageUtils"],
                path: "Sources/Vision/OCR",
                swiftSettings: [.define("ENABLE_VISION_BRIDGES")]),

        .target(name: "Text.Tokenization", dependencies: ["Core"], path: "Sources/Text/LanguageModels/Tokenization",
                swiftSettings: [.define("ENABLE_TEXT_BRIDGES")]),
        .target(name: "Text.LLM.Mistral", dependencies: ["Core","Text.Tokenization"],
                path: "Sources/Text/LanguageModels/Mistral",
                swiftSettings: [.define("ENABLE_TEXT_BRIDGES")]),
        .target(name: "Text.Generation.Diffusion", dependencies: ["Core"],
                path: "Sources/Text/Generation/Diffusion",
                swiftSettings: [.define("ENABLE_TEXT_BRIDGES")]),

        .target(name: "System.ModelMgmt", dependencies: ["Core"], path: "Sources/System/ModelManagement",
                swiftSettings: [.define("ENABLE_SYSTEM_BRIDGES")]),
        .target(name: "System.Perf", dependencies: ["Core"], path: "Sources/System/Performance",
                swiftSettings: [.define("ENABLE_SYSTEM_BRIDGES")]),

        // Swift ABI shim that exports C symbols directly
        .target(name: "BridgesFFI",
                dependencies: ["Core","System.ModelMgmt",
                               "Audio.STT.Whisper","Audio.STT.SpeechFramework",
                               "Vision.OD.YOLO","Vision.OCR.VisionOCR",
                               "Text.LLM.Mistral","Text.Generation.Diffusion"],
                path: "Sources/FFI",
                publicHeadersPath: "include"),

        .target(name: "AgentBridges",
                dependencies: ["Core",
                               "Audio.STT.Whisper","Audio.STT.SpeechFramework","Audio.Utils",
                               "Vision.OD.YOLO","Vision.OCR.VisionOCR","Vision.ImageUtils",
                               "Text.LLM.Mistral","Text.Tokenization","Text.Generation.Diffusion",
                               "System.ModelMgmt","System.Perf"],
                path: "Sources/AgentBridges"),

        .testTarget(name: "AudioTests", dependencies: ["Audio.STT.Whisper","Audio.STT.SpeechFramework"], path: "Tests/AudioTests"),
        .testTarget(name: "VisionTests", dependencies: ["Vision.OD.YOLO","Vision.OCR.VisionOCR"], path: "Tests/VisionTests"),
        .testTarget(name: "TextTests", dependencies: ["Text.LLM.Mistral","Text.Tokenization"], path: "Tests/TextTests"),
        .testTarget(name: "IntegrationTests", dependencies: ["BridgesFFI"], path: "Tests/IntegrationTests"),
        .testTarget(name: "FFIValidationTests", dependencies: ["BridgesFFI"], path: "Tests/FFIValidationTests"),
        .testTarget(name: "EndToEndTests", dependencies: ["BridgesFFI"], path: "Tests/EndToEndTests")
    ]
)
