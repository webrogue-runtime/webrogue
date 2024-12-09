cd $(dirname $0)
set -e

CMDLINE_TOOLS_VERSION=linux-11076708_latest
NDK_VERSION=26.1.10909125

test -f commandlinetools-$CMDLINE_TOOLS_VERSION.zip || wget https://dl.google.com/android/repository/commandlinetools-$CMDLINE_TOOLS_VERSION.zip
test -d cmdline-tools || unzip commandlinetools-$CMDLINE_TOOLS_VERSION.zip

export ANDROID_HOME="$(pwd)/android_sdk"
export ANDROID_NDK_PATH="$ANDROID_HOME/ndk/$NDK_VERSION"
test -d "$ANDROID_HOME/licenses" || yes | ./cmdline-tools/bin/sdkmanager --licenses --sdk_root=android_sdk
test -d "$ANDROID_NDK_PATH" || ./cmdline-tools/bin/sdkmanager --sdk_root=android_sdk "ndk;$NDK_VERSION"

../android/gradlew --project-dir=../android :runner:assembleRelease

rm -rf ../aot_artifacts/android_gradle
mkdir -p ../aot_artifacts/android_gradle
mkdir -p ../aot_artifacts/android_gradle/template

TARGET_FILES=$(cat android_gradle_files.txt)

for TARGET_FILE in $TARGET_FILES; do
    TARGET_DIR=$(dirname $TARGET_FILE)
    mkdir -p ../aot_artifacts/android_gradle/template/$TARGET_DIR
    cp ../android_template/$TARGET_FILE ../aot_artifacts/android_gradle/template/$TARGET_FILE
done

mkdir -p ../aot_artifacts/android_gradle/template/app/src/main/java/org/libsdl/app/
cp \
    ../android/common/src/main/java/org/libsdl/app/*.java \
    ../aot_artifacts/android_gradle/template/app/src/main/java/org/libsdl/app/

mkdir -p ../aot_artifacts/android_gradle/template/app/src/main/java/io/github/webrogue_runtime/common
cp \
    ../android/common/src/main/java/io/github/webrogue_runtime/common/WebrogueActivity.java \
    ../aot_artifacts/android_gradle/template/app/src/main/java/io/github/webrogue_runtime/common

mkdir -p ../aot_artifacts/android_gradle/template/app/src/main/java/io/github/webrogue_runtime/runner
cp \
    ../android/runner/src/main/java/io/github/webrogue_runtime/runner/WebrogueRunnerActivity.java \
    ../aot_artifacts/android_gradle/template/app/src/main/java/io/github/webrogue_runtime/runner


mkdir -p ../aot_artifacts/android_gradle/template/app/src/main/jniLibs/arm64-v8a/
cp \
    ../android/runner/build/intermediates/cxx/RelWithDebInfo/*/obj/arm64-v8a/libSDL2.so \
    ../aot_artifacts/android_gradle/template/app/src/main/jniLibs/arm64-v8a/libSDL2.so
mkdir -p ../aot_artifacts/android_gradle/libs
cp \
    ../android/runner/.cxx/RelWithDebInfo/*/arm64-v8a/webrogue_runner_common/CMakeFiles/webrogue.dir/webrogue_runtime.c.o \
    ../android/runner/src/main/cpp/../rust_target/aarch64-linux-android/release-lto/libwebrogue_android.a \
    ../aot_artifacts/android_gradle/libs

$ANDROID_NDK_PATH/toolchains/llvm/prebuilt/*/bin/llvm-ar qs \
    ../aot_artifacts/android_gradle/libs/libwebrogue_android.a \
    ../android/runner/.cxx/RelWithDebInfo/*/arm64-v8a/webrogue_runner_common/CMakeFiles/webrogue.dir/home/someone/repos/webrogue/crates/gfx-fallback/webrogue_gfx_ffi_sdl2.c.o

cp \
    ../android/gradlew \
    ../android/gradlew.bat \
    ../aot_artifacts/android_gradle/template

cp -r \
    ../android/gradle/wrapper \
    ../aot_artifacts/android_gradle/template/gradle/wrapper
