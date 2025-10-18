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
            targets: ["CoreMLBridge"]
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
        )
    ]
)
