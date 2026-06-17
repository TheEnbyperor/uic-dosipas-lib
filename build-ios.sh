#!/usr/bin/env bash

cargo build --lib --release --target x86_64-apple-ios
cargo build --lib --release --target aarch64-apple-ios-sim
cargo build --lib --release --target aarch64-apple-ios
cargo build --lib --release --target aarch64-apple-darwin

cargo run --bin uniffi-bindgen generate src/uic-dosipas.udl --config uniffi.toml --language swift --out-dir swift

mkdir -p target/uniffi-xcframework-staging/
mv swift/uic_dosipasFFI.h target/uniffi-xcframework-staging/uic_dosipasFFI.h
mv swift/uic_dosipasFFI.modulemap target/uniffi-xcframework-staging/module.modulemap

mkdir -p target/ios-simulator-fat/release
lipo -create target/x86_64-apple-ios/release/libuic_dosipas.a target/aarch64-apple-ios-sim/release/libuic_dosipas.a -output target/ios-simulator-fat/release/libuic_dosipas.a

rm -rf target/ios
xcodebuild -create-xcframework \
  -library target/aarch64-apple-ios/release/libuic_dosipas.a -headers target/uniffi-xcframework-staging \
  -library target/ios-simulator-fat/release/libuic_dosipas.a -headers target/uniffi-xcframework-staging \
  -library target/aarch64-apple-darwin/release/libuic_dosipas.a -headers target/uniffi-xcframework-staging \
  -output target/ios/libuic_dosipas-rs.xcframework

ditto -c -k --sequesterRsrc --keepParent target/ios/libuic_dosipas-rs.xcframework target/ios/libuic_dosipas-rs.xcframework.zip
checksum=$(swift package compute-checksum target/ios/libuic_dosipas-rs.xcframework.zip)
version=$(cargo metadata --format-version 1 | jq -r --arg pkg_name "uic-dosipas-lib" '.packages[] | select(.name==$pkg_name) .version')
sed -i "" -E "s/(let releaseTag = \")[^\"]+(\")/\1$version\2/g" Package.swift
sed -i "" -E "s/(let releaseChecksum = \")[^\"]+(\")/\1$checksum\2/g" Package.swift