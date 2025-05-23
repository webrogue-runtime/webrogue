set -ex

VARIANT=$1
test -f /opt/homebrew/bin/brew && eval "$(/opt/homebrew/bin/brew shellenv)"
CARGO_PATH=$(dirname $(whereis -q cargo) | echo $HOME/.cargo/bin)

XCODE_PATH="$CARGO_PATH:$PATH"
MODIFIED_PATH="$CARGO_PATH:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin"

# SDKROOT env variable somehow breaks build for ios
unset SDKROOT

cd $(dirname $0)
IOS_ROOT_DIR=$(pwd)

case "$CARGO_PROFILE" in
    debug)
        FLAGS_CONFIG=""
        ;;
    release)
        FLAGS_CONFIG="--release"
        ;;
    *)
        FLAGS_CONFIG="--profile=$CARGO_PROFILE"
        ;;
esac

case "$VARIANT" in
    runner)
        ;;
    runtime)
        ;;
    *)
        echo "error: unknown VARIANT: $VARIANT"
        exit 1
        ;;
esac

CARGO_ARCHS=""

LIPO_PATHS=()
LIPO_PATHS_I=0

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
    CARGO_TARGET_DIR=$BUILT_PRODUCTS_DIR/rust_target cargo build $FLAGS_CONFIG --target=$CARGO_TARGET --features=$VARIANT
    export PATH="$XCODE_PATH"
    LIPO_PATHS[$LIPO_PATHS_I]="$BUILT_PRODUCTS_DIR/rust_target/$CARGO_TARGET/$CARGO_PROFILE/libwebrogue_macos.a"
    LIPO_PATHS_I=$(expr $LIPO_PATHS_I '+' 1)
done

mkdir -p "$BUILD_DIR/rust_artifacts/$VARIANT/$CONFIGURATION/$PLATFORM_NAME"
lipo -create "${LIPO_PATHS[@]}" -output "$BUILD_DIR/rust_artifacts/$VARIANT/$CONFIGURATION/$PLATFORM_NAME/libwebrogue_macos.a"
