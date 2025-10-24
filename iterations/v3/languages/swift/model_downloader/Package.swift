// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "ModelDownloader",
    platforms: [
        .macOS(.v13)
    ],
    dependencies: [
        .package(url: "https://github.com/argmaxinc/whisperkit", from: "0.14.0"),
    ],
    targets: [
        .executableTarget(
            name: "ModelDownloader",
            dependencies: [
                .product(name: "WhisperKit", package: "whisperkit"),
            ]
        )
    ]
)
