// swift-tools-version: 5.10

import PackageDescription

let releaseTag = "0.0.1"
let releaseChecksum = "d211e5225de95ac74e3c35a8fc5e4398b2ba76d31a407acd6b7683935a0e56e2"
let binaryTarget: Target = .binaryTarget(
    name: "UicDosipasRS",
    url: "https://github.com/TheEnbyperor/uic-dosipas-lib/releases/download/\(releaseTag)/libuic_dosipas-rs.xcframework.zip",
    checksum: releaseChecksum
)

let package = Package(
    name: "uic-dosipas-lib",
    platforms: [
        .iOS(.v16),
        .macOS(.v12)
    ],
    products: [
        .library(
            name: "UicDosipas",
            targets: ["UicDosipas"]
        ),
    ],
    targets: [
        binaryTarget,
        .target(
            name: "UicDosipas",
            dependencies: ["UicDosipasRS"],
            path: "swift"
        ),
    ]
)