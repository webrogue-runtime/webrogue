XCODE_PATH="$HOME/.cargo/bin:$PATH"
MODIFIED_PATH="$HOME/.cargo/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin"

# SDKROOT env variable somehow breaks build for ios
unset SDKROOT

cd $(dirname $(dirname $0))
IOS_ROOT_DIR=$(pwd)
set -ex

case "$CONFIGURATION" in
    Debug)
        FLAGS_CONFIG=""
        CARGO_CONFIG_NAME="debug"
        ;;
    Release)
        FLAGS_CONFIG="--release"
        CARGO_CONFIG_NAME="release"
        ;;
    *)
        echo "error: unknown CONFIGURATION: $CONFIGURATION"
        exit 1
        ;;
esac

CARGO_ARCHS=""

LIPO_PATHS=""

for DEST_ARCH in $ARCHS; do
    case "$PLATFORM_NAME" in
        macosx)
            case "$DEST_ARCH" in
                x86_64)
                    CARGO_TARGET="x86_64-apple-darwin"
                    ;;
                arm64)
                    CARGO_TARGET="aarch64-apple-darwin"
                    ;;
                *)
                    echo "error: unknown DEST_ARCH: $DEST_ARCH"
                    exit 1
                    ;;
            esac
            ;;
        *)
            echo "error: unknown PLATFORM_NAME: $PLATFORM_NAME"
            exit 1
            ;;
    esac

    # cargo can't compile C sources when Xcode's PATH is active
    export PATH="$MODIFIED_PATH"
    CARGO_TARGET_DIR=$BUILT_PRODUCTS_DIR/rust_target cargo build $FLAGS_CONFIG --target=$CARGO_TARGET
    export PATH="$XCODE_PATH"
    LIPO_PATHS="$LIPO_PATHS $BUILT_PRODUCTS_DIR/rust_target/$CARGO_TARGET/$CARGO_CONFIG_NAME/libwebrogue_macos.a"
done

mkdir -p $BUILD_DIR/rust_artifacts/$CARGO_CONFIG_NAME/$PLATFORM_NAME
lipo -create $LIPO_PATHS -output $BUILD_DIR/rust_artifacts/$CONFIGURATION/$PLATFORM_NAME/libwebrogue_macos.a
