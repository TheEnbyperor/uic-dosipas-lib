// swift-tools-version: 5.10

import PackageDescription

let releaseTag = "0.0.3"
let releaseChecksum = "7e93dde014a4a0500248eae762cbfa5bd52e203992794fd09155783cfb350449"
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