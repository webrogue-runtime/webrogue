cd $(dirname $0) # android/runtime
set -ex

cd .. # android
. ./setup_sdk.sh
cd runtime # android/runtime

export ANDROID_HOME="$ANDROID_SDK_ROOT"
export ANDROID_NDK_HOME="$ANDROID_NDK_PATH"
export CARGO_NDK_PLATFORM=$ANDROID_API_VERSION

WRAPP_PATH=../../examples/glfw/glfw.wrapp
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
        --features=compile,llvm \
        compile \
        android-so \
        "$WRAPP_PATH" \
        runner/src/main/jniLibs/$TARGET/libwebrogue_aot.so \
        $RUST_TARGET-linux-android

    rustup target add $RUST_TARGET-linux-android

    $STRACE_COMMAND cargo ndk \
        -t $TARGET \
        -o runner/src/main/jniLibs/ \
        build \
        --features runner \
        --profile $1

    if [ "$BUILD_WEBROGUE_ANDROID_LAUNCHER" = "1" ]; then
        $STRACE_COMMAND cargo ndk \
            -t $TARGET \
            -o launcher/src/main/jniLibs/ \
            build \
            --features launcher \
            --profile $1

        $STRACE_COMMAND cargo ndk \
            -t $TARGET \
            --manifest-path launcher/Cargo.toml \
            -o launcher/src/main/jniLibs/ \
            build \
            --profile $1
    fi
done
