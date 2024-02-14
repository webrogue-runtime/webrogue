cd $(dirname $0)

set -ex

sh configure.sh
rm -rf ../../artifacts


CARGO_RELEASE_FLAG=
CARGO_BUILD_TYPE=debug
# CARGO_TARGETS=
CARGO_TARGETS="aarch64-apple-ios"
XCODE_CONFIGURATION_FLAG="-configuration=Debug"
XCODE_DESTINATION_FLAG_1=
XCODE_DESTINATION_FLAG_2=-parallelizeTargets

set_simulator_arch() {
    case "$(uname -m)" in
        x86_64)
            CARGO_TARGETS="x86_64-apple-ios"
            ;;
        arm64)
            CARGO_TARGETS="aarch64-apple-ios-sim"
            ;;
        *)
            echo "unknown arch: $(uname -m)"
            exit 1
            ;;
    esac
}

while test $# -gt 0; do
    case "$1" in
        -h|--help)
            echo "There is no help"
            exit 0
            ;;
        -s)
            set_simulator_arch
            shift
            if test $# -gt 0; then
                SIMULATOR_NAME=$1
            else
                echo "simulator name not specified"
                exit 1
            fi
            shift
            echo "arg: Build for simulator $SIMULATOR_NAME"
            XCODE_DESTINATION_FLAG_1="-destination"
            XCODE_DESTINATION_FLAG_2="platform=iOS Simulator,name=$SIMULATOR_NAME"
            ;;
        --simulator*)
            set_simulator_arch
            SIMULATOR_NAME=`echo $1 | sed -e 's/^[^=]*=//g'`
            shift
            echo "arg: Build for simulator $SIMULATOR_NAME"
            XCODE_DESTINATION_FLAG_1="-destination"
            XCODE_DESTINATION_FLAG_2="platform=iOS Simulator,name=$SIMULATOR_NAME"
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
    rm rust-toolchain || true

    cd lib/c-api
    # rustup target add aarch64-apple-darwin
    # cargo install cargo-lipo
    CARGO_TARGET_DIR=../../../../platforms/iOS/rust_target cargo lipo $CARGO_RELEASE_FLAG --targets=$CARGO_TARGETS --no-default-features --features jsc 
    cd ../../../../platforms/iOS
    mv rust_target/universal/$CARGO_BUILD_TYPE/libwasmer.a rust_target/universal/libwasmer.a
}

echo Building xcode project
{
    echo Building rust lib
    XCODEBUILD_FLAGS="-project cmake_build/webrogue_ios.xcodeproj -scheme webrogue $XCODE_CONFIGURATION_FLAG CODE_SIGN_IDENTITY=\"\" CODE_SIGNING_REQUIRED=NO CODE_SIGNING_ALLOWED=NO $XCODE_DESTINATION_FLAG_1"

    xcodebuild $XCODEBUILD_FLAGS "$XCODE_DESTINATION_FLAG_2"
    XC_BUILD_DIR=$(xcodebuild $XCODEBUILD_FLAGS "$XCODE_DESTINATION_FLAG_2" -showBuildSettings | grep -m 1 "BUILT_PRODUCTS_DIR" | grep -oEi "\/.*" || exit 3)
    mkdir ../../artifacts
    cp -r $XC_BUILD_DIR/Webrogue.app ../../artifacts
}