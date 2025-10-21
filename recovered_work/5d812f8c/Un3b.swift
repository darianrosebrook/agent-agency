// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "VisionBridge",
    platforms: [
        .macOS(.v13)
    ],
    products: [
        .library(
            name: "VisionBridge",
            targets: ["VisionBridge"]
        )
    ],
    dependencies: [],
    targets: [
        .target(
            name: "VisionBridge",
            dependencies: []
        )
    ]
)
