// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "ASRBridge",
    platforms: [
        .macOS(.v13)
    ],
    products: [
        .library(
            name: "ASRBridge",
            targets: ["ASRBridge"]
        )
    ],
    dependencies: [],
    targets: [
        .target(
            name: "ASRBridge",
            dependencies: []
        )
    ]
)
