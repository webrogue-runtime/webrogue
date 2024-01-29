cd $(dirname $0)

set -ex

sh configure.sh
rm -rf ../../artifacts

CARGO_RELEASE_FLAG=
CARGO_BUILD_TYPE=debug
# CARGO_TARGETS=
case "$(uname -m)" in
    x86_64)
        CARGO_TARGETS="x86_64-apple-darwin"
        ;;
    arm64)
        CARGO_TARGETS="aarch64-apple-darwin"
        ;;
    *)
        echo "unknown arch: $(uname -m)"
        exit 1
        ;;
esac
XCODE_CONFIGURATION_FLAG="-configuration=Debug"
XCORE_DESTINATION_FLAG=

while test $# -gt 0; do
    case "$1" in
        -h|--help)
            echo "There is no help"
            exit 0
            ;;
        -u|--universal)
            echo "arg: Univarsal build"
            CARGO_TARGETS="x86_64-apple-darwin,aarch64-apple-darwin"
            XCORE_DESTINATION_FLAG="-destination generic/platform=macOS"
            shift
            ;;
        -r|--release)
            echo "arg: Release build"
            XCODE_CONFIGURATION_FLAG="-configuration=Release"
            CARGO_BUILD_TYPE=release
            CARGO_RELEASE_FLAG="--release"
            shift
            ;;
        *)
            echo "arg: Unknown. Exitting"
            exit 1
            ;;
    esac
done

echo Building rust lib
{
    cd ../../external/wasmer/
    git apply ../wasmer.patch || true

    cd lib/c-api
    # rustup target add aarch64-apple-darwin
    # cargo install cargo-lipo
    CARGO_TARGET_DIR=../../../../platforms/MacOS/rust_target cargo lipo $CARGO_RELEASE_FLAG --targets=$CARGO_TARGETS --no-default-features --features jsc 
    cd ../../../../platforms/MacOS
    mv rust_target/universal/$CARGO_BUILD_TYPE/libwasmer.a rust_target/universal/libwasmer.a
}

echo Building xcode project
{
    echo Building rust lib
    XCODEBUILD_FLAGS="-project cmake_build/webrogue_macos.xcodeproj -scheme webrogue $XCODE_CONFIGURATION_FLAG $XCORE_DESTINATION_FLAG"

    xcodebuild $XCODEBUILD_FLAGS -parallelizeTargets
    XC_BUILD_DIR=$(xcodebuild $XCODEBUILD_FLAGS -showBuildSettings | grep -m 1 "BUILT_PRODUCTS_DIR" | grep -oEi "\/.*" || exit 3)
    mkdir ../../artifacts
    cp -r $XC_BUILD_DIR/Webrogue.app ../../artifacts
}