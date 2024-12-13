cd $(dirname $0)
set -e

cd ../apple
cp ../examples/raylib/raylib.webc macOS/runner/aot.webc

sh setup.command

# cargo run --manifest-path ../crates/aot-compiler/Cargo.toml object macOS/runner/aot.webc macOS/runner/aot_x86_64.o x86_64-apple-darwin
# cargo run --manifest-path ../crates/aot-compiler/Cargo.toml object macOS/runner/aot.webc macOS/runner/aot_arm64.o arm64-apple-darwin
XC_FLAGS="-destination generic/platform=macOS -workspace webrogue.xcworkspace -scheme MacOS_Runner_ReleaseLocal -configuration ReleaseLocal"
XC_BUILT_PRODUCTS_DIR=$(xcodebuild $XC_FLAGS -showBuildSettings | grep -m 1 "BUILT_PRODUCTS_DIR =" | grep -oEi "\/.*" || exit 3)
XC_BUILD_DIR=$(xcodebuild $XC_FLAGS -showBuildSettings | grep -m 1 "BUILD_DIR =" | grep -oEi "\/.*" || exit 3)
xcodebuild $XC_FLAGS -parallelizeTargets -allowProvisioningUpdates

cd ../apple_template


rm -rf bin
mkdir -p bin
cp $XC_BUILD_DIR/rust_artifacts/runner/ReleaseLocal/macosx/libwebrogue_macos.a bin/libwebrogue_macos.macosx.a
cp $XC_BUILD_DIR/Release/libSDL2.a bin/libSDL2.macosx.a
cp $XC_BUILD_DIR/ReleaseLocal/libEGL.dylib bin/libEGL.macosx.dylib
cp $XC_BUILD_DIR/ReleaseLocal/libGLESv2.dylib bin/libGLESv2.macosx.dylib

rm -rf aot
mkdir -p aot
cp ../apple/macOS/runner/aot_arm64.o aot/aot_arm64.o
cp ../apple/macOS/runner/aot_x86_64.o aot/aot_x86_64.o
cp ../apple/macOS/runner/aot.webc aot/aot.webc

cp ../apple/scripts/lipo_object_combiner.sh scripts/lipo_object_combiner.sh
cp ../apple/macOS/runner/runner.entitlements macos/runner.entitlements

xcodegen

rm -rf ../aot_artifacts/apple_xcode
mkdir -p ../aot_artifacts/apple_xcode
mkdir -p ../aot_artifacts/apple_xcode/template

TARGET_FILES=$(cat template_files.txt)

for TARGET_FILE in $TARGET_FILES; do
    TARGET_DIR=$(dirname $TARGET_FILE)
    mkdir -p ../aot_artifacts/apple_xcode/template/$TARGET_DIR
    cp ../apple_template/$TARGET_FILE ../aot_artifacts/apple_xcode/template/$TARGET_FILE
done
