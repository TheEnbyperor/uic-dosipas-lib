// swift-tools-version: 5.10

import PackageDescription

let releaseTag = "0.0.5"
let releaseChecksum = "9d0dd4e61f961ad9c901ec64885e97622f6a89dc9ce236b4308540fd5ee24c74"
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