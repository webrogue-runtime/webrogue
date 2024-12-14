# apple/template
cd $(dirname $0)
set -e
# apple/runtime
cd ../runtime
cp ../../examples/raylib/raylib.webc macOS/runner/aot.webc

sh setup.command

cargo run --release --manifest-path ../../crates/aot-compiler/Cargo.toml object macOS/runner/aot.webc macOS/runner/aot_x86_64.o x86_64-apple-darwin
cargo run --release --manifest-path ../../crates/aot-compiler/Cargo.toml object macOS/runner/aot.webc macOS/runner/aot_arm64.o arm64-apple-darwin
XC_FLAGS="-destination generic/platform=macOS -workspace webrogue.xcworkspace -scheme MacOS_Runner_ReleaseLocal -configuration ReleaseLocal"
XC_BUILT_PRODUCTS_DIR=$(xcodebuild $XC_FLAGS -showBuildSettings | grep -m 1 "BUILT_PRODUCTS_DIR =" | grep -oEi "\/.*" || exit 3)
XC_BUILD_DIR=$(xcodebuild $XC_FLAGS -showBuildSettings | grep -m 1 "BUILD_DIR =" | grep -oEi "\/.*" || exit 3)
xcodebuild $XC_FLAGS -parallelizeTargets -allowProvisioningUpdates

# apple
cd ..

rm -rf template/bin
mkdir -p template/bin
cp $XC_BUILD_DIR/rust_artifacts/runner/ReleaseLocal/macosx/libwebrogue_macos.a template/bin/libwebrogue_macos.macosx.a
cp $XC_BUILD_DIR/Release/libSDL2.a template/bin/libSDL2.macosx.a
cp $XC_BUILD_DIR/ReleaseLocal/libEGL.dylib template/bin/libEGL.macosx.dylib
cp $XC_BUILD_DIR/ReleaseLocal/libGLESv2.dylib template/bin/libGLESv2.macosx.dylib

rm -rf template/aot
mkdir -p template/aot
cp runtime/macOS/runner/aot_arm64.o template/aot/aot_arm64.o
cp runtime/macOS/runner/aot_x86_64.o template/aot/aot_x86_64.o
cp runtime/macOS/runner/aot.webc template/aot/aot.webc

cp runtime/scripts/lipo_object_combiner.sh template/scripts/lipo_object_combiner.sh
cp runtime/macOS/runner/runner.entitlements template/macos/runner.entitlements

# apple/template
cd template
xcodegen

rm -rf ../../aot_artifacts/apple_xcode
mkdir -p ../../aot_artifacts/apple_xcode
mkdir -p ../../aot_artifacts/apple_xcode/template

TARGET_FILES=$(cat template_files.txt)

for TARGET_FILE in $TARGET_FILES; do
    TARGET_DIR=$(dirname $TARGET_FILE)
    mkdir -p ../../aot_artifacts/apple_xcode/template/$TARGET_DIR
    cp ../../apple/template/$TARGET_FILE ../../aot_artifacts/apple_xcode/template/$TARGET_FILE
done
