// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "CoreMLBridge",
    platforms: [
        .macOS(.v12)
    ],
    products: [
        .library(
            name: "CoreMLBridge",
            targets: ["CoreMLBridge", "WhisperAudio", "MistralTokenizer"]
        )
    ],
    targets: [
        .target(
            name: "CoreMLBridge",
            dependencies: [],
            linkerSettings: [
                .linkedFramework("CoreML"),
                .linkedFramework("Foundation")
            ]
        ),
        .target(
            name: "WhisperAudio",
            dependencies: [],
            linkerSettings: [
                .linkedFramework("CoreML"),
                .linkedFramework("Foundation"),
                .linkedFramework("Accelerate"),
                .linkedFramework("AVFoundation")
            ]
        ),
        .target(
            name: "MistralTokenizer",
            dependencies: [],
            linkerSettings: [
                .linkedFramework("Foundation"),
                .linkedFramework("NaturalLanguage")
            ]
        )
    ]
)
