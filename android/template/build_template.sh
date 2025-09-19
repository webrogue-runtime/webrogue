cd $(dirname $0) # android/template
set -e

# android
cd ..
CMDLINE_TOOLS_VERSION=linux-11076708_latest
NDK_VERSION=27.3.13750724
ANDROID_API_VERSION=24

if test -z $ANDROID_SDK_ROOT
then
    export ANDROID_SDK_ROOT="$(pwd)/sdk"
    test -d $ANDROID_SDK_ROOT || mkdir $ANDROID_SDK_ROOT
fi
test -d $ANDROID_SDK_ROOT/cmdline-tools || {
    wget https://dl.google.com/android/repository/commandlinetools-$CMDLINE_TOOLS_VERSION.zip -O $ANDROID_SDK_ROOT/commandlinetools-$CMDLINE_TOOLS_VERSION.zip
    unzip $ANDROID_SDK_ROOT/commandlinetools-$CMDLINE_TOOLS_VERSION.zip -d $ANDROID_SDK_ROOT
    rm $ANDROID_SDK_ROOT/commandlinetools-$CMDLINE_TOOLS_VERSION.zip
}

export ANDROID_NDK_PATH="$ANDROID_SDK_ROOT/ndk/$NDK_VERSION"
test -d "$ANDROID_SDK_ROOT/licenses" || yes | $ANDROID_SDK_ROOT/cmdline-tools/bin/sdkmanager --licenses --sdk_root=$ANDROID_SDK_ROOT
test -d "$ANDROID_NDK_PATH" || $ANDROID_SDK_ROOT/cmdline-tools/bin/sdkmanager --sdk_root=$ANDROID_SDK_ROOT "ndk;$NDK_VERSION"

rm -f runtime/runner/.cxx/RelWithDebInfo/*/arm64-v8a/webrogue_runner_common/CMakeFiles/webrogue.dir/home/someone/repos/webrogue/crates/gfx-fallback/*
rm -f runtime/runner/build/intermediates/cxx/RelWithDebInfo/*/obj/arm64-v8a/libSDL3.so 
rm -f runtime/runner/.cxx/RelWithDebInfo/*/arm64-v8a/webrogue_runner_common/CMakeFiles/webrogue.dir/webrogue_runtime.c.o

test -f ../examples/empty/empty.wrapp && {
    cargo run \
        --target-dir=../target \
        --release \
        --no-default-features \
        --features=compile \
        compile \
        object \
        ../examples/empty/empty.wrapp \
        runtime/runner/src/main/cpp/empty.o \
        aarch64-linux-android

    mkdir -p runtime/runner/src/main/assets
    cp ../examples/empty/empty.wrapp runtime/runner/src/main/assets/aot.swrapp # TODO strip
}

sh runtime/common/src/main/cpp/external/download_sdl.sh

rm -f process_dump/p.*
STRACE_COMMAND="strace -s 1000 -o process_dump/p -ff"
STRACE_COMMAND=""
$STRACE_COMMAND ./runtime/gradlew --no-daemon --project-dir=runtime :runner:assembleRelease

rm -rf ../aot_artifacts/android_gradle
mkdir -p ../aot_artifacts/android_gradle
mkdir -p ../aot_artifacts/android_gradle/template

TARGET_FILES=$(cat template/template_files.txt)

for TARGET_FILE in $TARGET_FILES; do
    TARGET_DIR=$(dirname $TARGET_FILE)
    mkdir -p ../aot_artifacts/android_gradle/template/$TARGET_DIR
    cp ./template/$TARGET_FILE ../aot_artifacts/android_gradle/template/$TARGET_FILE
done
echo "*" > ../aot_artifacts/android_gradle/template/.gitignore

mkdir -p ../aot_artifacts/android_gradle/template/app/src/main/java/org/libsdl/app/
cp \
    runtime/common/src/main/java/org/libsdl/app/*.java \
    ../aot_artifacts/android_gradle/template/app/src/main/java/org/libsdl/app/

mkdir -p ../aot_artifacts/android_gradle/template/app/src/main/java/io/github/webrogue_runtime/common
cp \
    runtime/common/src/main/java/io/github/webrogue_runtime/common/WebrogueActivity.java \
    ../aot_artifacts/android_gradle/template/app/src/main/java/io/github/webrogue_runtime/common

mkdir -p ../aot_artifacts/android_gradle/template/app/src/main/java/io/github/webrogue_runtime/runner
cp \
    runtime/runner/src/main/java/io/github/webrogue_runtime/runner/WebrogueRunnerActivity.java \
    ../aot_artifacts/android_gradle/template/app/src/main/java/io/github/webrogue_runtime/runner


mkdir -p ../aot_artifacts/android_gradle/template/app/src/main/jniLibs/arm64-v8a/
cp \
    runtime/runner/build/intermediates/cxx/RelWithDebInfo/*/obj/arm64-v8a/libSDL3.so \
    ../aot_artifacts/android_gradle/template/app/src/main/jniLibs/arm64-v8a/libSDL3.so
mkdir -p ../aot_artifacts/android_gradle/libs
cp \
    runtime/runner/.cxx/RelWithDebInfo/*/arm64-v8a/webrogue_runner_common/CMakeFiles/webrogue.dir/webrogue_runtime.c.o \
    runtime/runner/src/main/cpp/../rust_target/aarch64-linux-android/aot/libwebrogue_android.a \
    $ANDROID_SDK_ROOT/ndk/$NDK_VERSION/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/aarch64-linux-android/$ANDROID_API_VERSION/libc.so \
    $ANDROID_SDK_ROOT/ndk/$NDK_VERSION/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/aarch64-linux-android/$ANDROID_API_VERSION/libdl.so \
    $ANDROID_SDK_ROOT/ndk/$NDK_VERSION/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/aarch64-linux-android/$ANDROID_API_VERSION/liblog.so \
    $ANDROID_SDK_ROOT/ndk/$NDK_VERSION/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/aarch64-linux-android/$ANDROID_API_VERSION/crtbegin_so.o \
    $ANDROID_SDK_ROOT/ndk/$NDK_VERSION/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/aarch64-linux-android/$ANDROID_API_VERSION/crtend_so.o \
    ../aot_artifacts/android_gradle/libs

$ANDROID_NDK_PATH/toolchains/llvm/prebuilt/*/bin/llvm-ar qLs \
    ../aot_artifacts/android_gradle/libs/libwebrogue_android.a \
    runtime/runner/.cxx/RelWithDebInfo/*/arm64-v8a/webrogue_runner_common/libwebrogue_static.a \
    $ANDROID_SDK_ROOT/ndk/$NDK_VERSION/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/aarch64-linux-android/libc++_static.a \
    $ANDROID_SDK_ROOT/ndk/$NDK_VERSION/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/aarch64-linux-android/libc++abi.a \
    $ANDROID_SDK_ROOT/ndk/$NDK_VERSION/toolchains/llvm/prebuilt/linux-x86_64/lib/clang/18/lib/linux/libclang_rt.builtins-aarch64-android.a \
    $ANDROID_SDK_ROOT/ndk/$NDK_VERSION/toolchains/llvm/prebuilt/linux-x86_64/lib/clang/18/lib/linux/aarch64/libunwind.a

cp \
    $ANDROID_SDK_ROOT/ndk/$NDK_VERSION/toolchains/llvm/prebuilt/linux-x86_64/sysroot/NOTICE \
    ../aot_artifacts/android_gradle/libs/NOTICE

cp \
    runtime/gradlew \
    runtime/gradlew.bat \
    ../aot_artifacts/android_gradle/template

mkdir -p ../aot_artifacts/android_gradle/template/gradle/wrapper
cp \
    runtime/gradle/wrapper/gradle-wrapper.jar \
    runtime/gradle/wrapper/gradle-wrapper.properties \
    ../aot_artifacts/android_gradle/template/gradle/wrapper

cat /dev/urandom | head -c 32 > ../aot_artifacts/android_gradle/template_id
