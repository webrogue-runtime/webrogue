cd $(dirname $0) # android/runtime
set -ex

cd .. # android
. ./setup_sdk.sh
cd runtime # android/runtime

export ANDROID_HOME="$ANDROID_SDK_ROOT"
export ANDROID_NDK_HOME="$ANDROID_NDK_PATH"
export CARGO_NDK_PLATFORM=$ANDROID_API_VERSION

WRAPP_PATH=../../webrogue-sdk/examples/empty/empty.wrapp
mkdir -p runner/src/main/assets
cp $WRAPP_PATH runner/src/main/assets/aot.swrapp
rm -rf runner/src/main/jniLibs

cargo install cargo-ndk

# CARGO_TARGET_VAR="-C link-arg=-Wl,--no-allow-shlib-undefined -C link-arg=-Wl,--no-undefined"
# export CARGO_TARGET_AARCH64_LINUX_ANDROID_RUSTFLAGS="$CARGO_TARGET_VAR"
# export CARGO_TARGET_X86_64_LINUX_ANDROID_RUSTFLAGS="$CARGO_TARGET_VAR"

# rm -f ../process_dump/p.*
# STRACE_COMMAND="strace -s 1000 -o ../process_dump/p -ff"

for TARGET in arm64-v8a x86_64; do
    if [ "$TARGET" = "arm64-v8a" ]; then
        RUST_TARGET=aarch64
    else
        RUST_TARGET="$TARGET"
    fi

    mkdir -p runner/src/main/jniLibs/$TARGET
    cargo \
        run \
        --manifest-path ../../Cargo.toml \
        --no-default-features \
        --features=internal-compile,llvm \
        internal compile android-so \
        "$WRAPP_PATH" \
        runner/src/main/jniLibs/$TARGET/libwebrogue_aot.so \
        $RUST_TARGET-linux-android

    rustup target add $RUST_TARGET-linux-android
    CARGO_FLAGS="--profile $1 -Z build-std"

    for VULKAN_TYPE in 'novulkan' 'vulkan'
    do
        if [ "$VULKAN_TYPE" = "vulkan" ]; then
            CARGO_VULKAN_FLAGS="--features=vulkan"
        else
            CARGO_VULKAN_FLAGS=""
        fi

        $STRACE_COMMAND cargo +nightly ndk \
            -t $TARGET \
            -o runner/src/main/jniLibs-$VULKAN_TYPE/ \
            build \
            --features runner \
            $CARGO_FLAGS \
            $CARGO_VULKAN_FLAGS
    done

    if [ "$BUILD_WEBROGUE_ANDROID_LAUNCHER" = "1" ]; then
        if [ "$TARGET" = "arm64-v8a" ]; then
            CARGO_VULKAN_FLAGS="--features=vulkan"
        else
            CARGO_VULKAN_FLAGS=""
        fi
        $STRACE_COMMAND cargo +nightly ndk \
            -t $TARGET \
            -o launcher/src/main/jniLibs/ \
            build \
            --features launcher \
            $CARGO_FLAGS \
            $CARGO_VULKAN_FLAGS

        $STRACE_COMMAND cargo +nightly ndk \
            -t $TARGET \
            --manifest-path launcher/Cargo.toml \
            -o launcher/src/main/jniLibs/ \
            build \
            $CARGO_FLAGS
    fi
done
cp runner/src/main/jniLibs-vulkan/arm64-v8a/libwebrogue.so runner/src/main/jniLibs/arm64-v8a 
cp runner/src/main/jniLibs-novulkan/x86_64/libwebrogue.so runner/src/main/jniLibs/x86_64
