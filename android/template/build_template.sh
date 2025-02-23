# android/template
cd $(dirname $0)
set -e

# android
cd ..
CMDLINE_TOOLS_VERSION=linux-11076708_latest
NDK_VERSION=27.2.12479018

test -d sdk || mkdir sdk
test -d sdk/cmdline-tools || {
    wget https://dl.google.com/android/repository/commandlinetools-$CMDLINE_TOOLS_VERSION.zip -O sdk/commandlinetools-$CMDLINE_TOOLS_VERSION.zip
    unzip sdk/commandlinetools-$CMDLINE_TOOLS_VERSION.zip -d sdk
    rm sdk/commandlinetools-$CMDLINE_TOOLS_VERSION.zip
}


export ANDROID_HOME="$(pwd)/sdk"
export ANDROID_NDK_PATH="$ANDROID_HOME/ndk/$NDK_VERSION"
test -d "$ANDROID_HOME/licenses" || yes | ./sdk/cmdline-tools/bin/sdkmanager --licenses --sdk_root=sdk
test -d "$ANDROID_NDK_PATH" || ./sdk/cmdline-tools/bin/sdkmanager --sdk_root=sdk "ndk;$NDK_VERSION"

rm -f runtime/runner/.cxx/RelWithDebInfo/*/arm64-v8a/webrogue_runner_common/CMakeFiles/webrogue.dir/home/someone/repos/webrogue/crates/gfx-fallback/webrogue_gfx_ffi_sdl2.c.o
rm -f runtime/runner/build/intermediates/cxx/RelWithDebInfo/*/obj/arm64-v8a/libSDL2.so 
rm -f runtime/runner/.cxx/RelWithDebInfo/*/arm64-v8a/webrogue_runner_common/CMakeFiles/webrogue.dir/webrogue_runtime.c.o
cargo run \
    --target-dir=../target \
    --release \
    --no-default-features \
    --package=webrogue-aot-compiler \
    object \
    ../examples/gears/gears.webc \
    runtime/runner/src/main/cpp/aot.o \
    aarch64-linux-android

./runtime/gradlew --project-dir=runtime :runner:assembleRelease

rm -rf ../aot_artifacts/android_gradle
mkdir -p ../aot_artifacts/android_gradle
mkdir -p ../aot_artifacts/android_gradle/template

TARGET_FILES=$(cat template/template_files.txt)

for TARGET_FILE in $TARGET_FILES; do
    TARGET_DIR=$(dirname $TARGET_FILE)
    mkdir -p ../aot_artifacts/android_gradle/template/$TARGET_DIR
    cp ./template/$TARGET_FILE ../aot_artifacts/android_gradle/template/$TARGET_FILE
done

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
    runtime/runner/build/intermediates/cxx/RelWithDebInfo/*/obj/arm64-v8a/libSDL2.so \
    ../aot_artifacts/android_gradle/template/app/src/main/jniLibs/arm64-v8a/libSDL2.so
mkdir -p ../aot_artifacts/android_gradle/libs
cp \
    runtime/runner/.cxx/RelWithDebInfo/*/arm64-v8a/webrogue_runner_common/CMakeFiles/webrogue.dir/webrogue_runtime.c.o \
    runtime/runner/src/main/cpp/../rust_target/aarch64-linux-android/release-lto/libwebrogue_android.a \
    ../aot_artifacts/android_gradle/libs

$ANDROID_NDK_PATH/toolchains/llvm/prebuilt/*/bin/llvm-ar qLs \
    ../aot_artifacts/android_gradle/libs/libwebrogue_android.a \
    runtime/runner/.cxx/RelWithDebInfo/*/arm64-v8a/webrogue_runner_common/libwebrogue_static.a

cp \
    runtime/gradlew \
    runtime/gradlew.bat \
    ../aot_artifacts/android_gradle/template

mkdir -p ../aot_artifacts/android_gradle/template/gradle/wrapper
cp \
    runtime/gradle/wrapper/gradle-wrapper.jar \
    runtime/gradle/wrapper/gradle-wrapper.properties \
    ../aot_artifacts/android_gradle/template/gradle/wrapper
