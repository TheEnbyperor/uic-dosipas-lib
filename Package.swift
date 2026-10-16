// swift-tools-version: 5.10

import PackageDescription

let releaseTag = "0.0.6"
let releaseChecksum = "f76511975d7359cdd910f24795d78efa85cff15822703d53ba770e54bfc6a072"
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