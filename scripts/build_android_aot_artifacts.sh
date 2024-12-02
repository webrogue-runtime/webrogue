cd $(dirname $0)
set -ex

CMDLINE_TOOLS_VERSION=linux-11076708_latest
NDK_VERSION=26.1.10909125

test -f commandlinetools-$CMDLINE_TOOLS_VERSION.zip || wget https://dl.google.com/android/repository/commandlinetools-$CMDLINE_TOOLS_VERSION.zip
test -d cmdline-tools || unzip commandlinetools-$CMDLINE_TOOLS_VERSION.zip

export ANDROID_HOME="$(pwd)/android_sdk"
export ANDROID_NDK_PATH="$ANDROID_HOME/ndk/$NDK_VERSION"
test -d "$ANDROID_HOME/licenses" || yes | ./cmdline-tools/bin/sdkmanager --licenses --sdk_root=android_sdk
test -d "$ANDROID_NDK_PATH" || ./cmdline-tools/bin/sdkmanager --sdk_root=android_sdk "ndk;$NDK_VERSION"

../android/gradlew --project-dir=../android :runner:assembleRelease

rm -rf ../aot_artifacts/android_nogradle
mkdir -p ../aot_artifacts/android_nogradle
mkdir ../aot_artifacts/android_nogradle/arm64-v8a
cp ../android/runner/build/outputs/apk/release/runner-release-unsigned.apk ../aot_artifacts/android_nogradle/runner.apk
zip -d ../aot_artifacts/android_nogradle/runner.apk lib/arm64-v8a/libwebrogue.so assets/aot.webc
cp \
    ../android/runner/.cxx/RelWithDebInfo/*/arm64-v8a/webrogue_runner_common/CMakeFiles/webrogue.dir/webrogue_runtime.cpp.o \
    ../android/runner/.cxx/RelWithDebInfo/*/arm64-v8a/webrogue_runner_common/CMakeFiles/webrogue.dir/home/someone/repos/webrogue/crates/gfx-fallback/webrogue_gfx_ffi_sdl2.c.o \
    ../android/runner/src/main/cpp/../rust_target/aarch64-linux-android/release-lto/libwebrogue_android.a \
    ../android/runner/build/intermediates/cxx/RelWithDebInfo/*/obj/arm64-v8a/libSDL2.so \
    $ANDROID_NDK_PATH/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/aarch64-linux-android/24/crtbegin_so.o \
    $ANDROID_NDK_PATH/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/aarch64-linux-android/24/libandroid.so \
    $ANDROID_NDK_PATH/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/aarch64-linux-android/24/liblog.so \
    $ANDROID_NDK_PATH/toolchains/llvm/prebuilt/linux-x86_64/lib/clang/17/lib/linux/aarch64/libatomic.a \
    $ANDROID_NDK_PATH/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/aarch64-linux-android/24/libm.so \
    $ANDROID_NDK_PATH/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/aarch64-linux-android/24/libc++.a \
    $ANDROID_NDK_PATH/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/aarch64-linux-android/libc++_static.a \
    $ANDROID_NDK_PATH/toolchains/llvm/prebuilt/linux-x86_64/lib/clang/17/lib/linux/libclang_rt.builtins-aarch64-android.a \
    $ANDROID_NDK_PATH/toolchains/llvm/prebuilt/linux-x86_64/lib/clang/17/lib/linux/aarch64/libunwind.a \
    $ANDROID_NDK_PATH/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/aarch64-linux-android/24/libc.so \
    $ANDROID_NDK_PATH/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/aarch64-linux-android/24/libdl.so \
    $ANDROID_NDK_PATH/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/aarch64-linux-android/libc++abi.a \
    $ANDROID_NDK_PATH/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/aarch64-linux-android/24/crtend_so.o \
    ../aot_artifacts/android_nogradle/arm64-v8a
    
rm -rf android_build
mkdir android_build
cp ../aot_artifacts/android_nogradle/runner.apk android_build/runner.apk
mkdir android_build/apk_dir
# unzip android_build/runner.apk -d android_build/apk_dir

mkdir android_build/apk_dir/lib
mkdir android_build/apk_dir/lib/arm64-v8a

cargo run \
    --target-dir=../target \
    --package=webrogue-aot-compiler \
    object \
    ../examples/raylib/raylib.webc \
    android_build/aot.o \
    aarch64-linux-android

ld.lld \
--sysroot=/toolchains/llvm/prebuilt/linux-x86_64/sysroot \
-EL \
--fix-cortex-a53-843419 \
-z now \
-z relro \
-z max-page-size=4096 \
--hash-style=gnu \
--eh-frame-hdr \
-m aarch64linux \
-shared \
-o android_build/apk_dir/lib/arm64-v8a/libwebrogue.so \
-L../aot_artifacts/android_nogradle/arm64-v8a/ \
../aot_artifacts/android_nogradle/arm64-v8a/crtbegin_so.o \
--build-id=sha1 \
--no-rosegment \
--no-undefined-version \
--fatal-warnings \
--no-undefined \
-soname libwebrogue.so \
../aot_artifacts/android_nogradle/arm64-v8a/webrogue_runtime.cpp.o \
../aot_artifacts/android_nogradle/arm64-v8a/webrogue_gfx_ffi_sdl2.c.o \
../aot_artifacts/android_nogradle/arm64-v8a/libandroid.so \
../aot_artifacts/android_nogradle/arm64-v8a/liblog.so \
../aot_artifacts/android_nogradle/arm64-v8a/libwebrogue_android.a \
../aot_artifacts/android_nogradle/arm64-v8a/libSDL2.so \
android_build/aot.o \
../aot_artifacts/android_nogradle/arm64-v8a/libatomic.a \
../aot_artifacts/android_nogradle/arm64-v8a/libc++.a \
../aot_artifacts/android_nogradle/arm64-v8a/libc++_static.a \
../aot_artifacts/android_nogradle/arm64-v8a/libm.so \
../aot_artifacts/android_nogradle/arm64-v8a/libc.so \
../aot_artifacts/android_nogradle/arm64-v8a/libclang_rt.builtins-aarch64-android.a \
../aot_artifacts/android_nogradle/arm64-v8a/libunwind.a \
../aot_artifacts/android_nogradle/arm64-v8a/libdl.so \
../aot_artifacts/android_nogradle/arm64-v8a/crtend_so.o

mkdir android_build/apk_dir/assets
cp ../examples/raylib/raylib.webc android_build/apk_dir/assets/aot.webc
cd android_build/apk_dir
zip ../runner.apk -0 lib/arm64-v8a/libwebrogue.so
zip ../runner.apk assets/aot.webc
cd ../..
rm -rf android_build/apk_dir
$ANDROID_HOME/build-tools/34.0.0/zipalign -p 4 android_build/runner.apk android_build/runner-alligned.apk
# keytool -genkey -v -keystore test.jks -alias alias_name -keyalg RSA -keysize 2048 -validity 10000
mv android_build/runner-alligned.apk android_build/runner.apk
$ANDROID_HOME/build-tools/34.0.0/apksigner sign --ks test.jks --ks-pass=pass:testtest android_build/runner.apk
