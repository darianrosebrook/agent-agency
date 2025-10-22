// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "CoreMLBridge",
    platforms: [
        .macOS(.v11)
    ],
    products: [
        .library(
            name: "CoreMLBridge",
            targets: ["CoreMLBridge", "WhisperAudio"]
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
        )
    ]
)
