cd $(dirname $0)
set -ex

sh scripts/download_angle_ios_headers.sh
sh scripts/download_angle_ios_xcframeworks.sh
sh scripts/download_sdl_src.sh

cp ../../examples/raylib/raylib.wrapp macos/runner/aot.wrapp
cp ../../examples/raylib/raylib.wrapp ios/runner/aot.wrapp

cargo run --release --no-default-features --manifest-path ../../crates/aot-compiler/Cargo.toml object macos/runner/aot.wrapp macos/runner/aot.x86_64.macosx.o x86_64-apple-darwin --pic
cargo run --release --no-default-features --manifest-path ../../crates/aot-compiler/Cargo.toml object macos/runner/aot.wrapp macos/runner/aot.arm64.macosx.o arm64-apple-darwin --pic
cargo run --release --no-default-features --manifest-path ../../crates/aot-compiler/Cargo.toml object ios/runner/aot.wrapp ios/runner/aot.x86_64.iphonesimulator.o x86_64-apple-ios --pic
cargo run --release --no-default-features --manifest-path ../../crates/aot-compiler/Cargo.toml object ios/runner/aot.wrapp ios/runner/aot.arm64.iphonesimulator.o arm64-apple-ios-sim --pic
cargo run --release --no-default-features --manifest-path ../../crates/aot-compiler/Cargo.toml object ios/runner/aot.wrapp ios/runner/aot.arm64.iphoneos.o arm64-apple-ios --pic

xcodegen -c
