// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "CoreMLBridge",
    platforms: [
        .macOS(.v13)
    ],
    products: [
        .library(
            name: "CoreMLBridge",
            targets: ["CoreMLBridge", "WhisperAudio", "MistralTokenizer", "YOLOImage"]
        )
    ],
    dependencies: [
        .package(url: "https://github.com/argmaxinc/whisperkit", from: "0.14.0")
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
            dependencies: [
                .product(name: "WhisperKit", package: "whisperkit")
            ],
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
        ),
        .target(
            name: "YOLOImage",
            dependencies: [],
            linkerSettings: [
                .linkedFramework("CoreML"),
                .linkedFramework("Foundation"),
                .linkedFramework("CoreGraphics"),
                .linkedFramework("Accelerate")
            ]
        )
    ]
)
