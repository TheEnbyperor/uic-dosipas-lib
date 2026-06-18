// swift-tools-version: 5.10

import PackageDescription

let releaseTag = "0.0.1"
let releaseChecksum = "bd1c0eab86c9e0cd73b7379ff098cb5815b4dbbe0ac6bab64c537e5f74e7bb09"
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