cd $(dirname $0) # android/template
set -e

# sh ../runtime/build_native_code.sh aot


cd .. # android
. ./setup_sdk.sh
# ./runtime/gradlew --no-daemon --project-dir=runtime :runner:assembleRelease

COMMON_OUT_PATH=../aot_artifacts/android
rm -rf $COMMON_OUT_PATH
mkdir -p $COMMON_OUT_PATH
mkdir -p $COMMON_OUT_PATH/template
COMMON_OUT_PATH="$(realpath $COMMON_OUT_PATH)"

rm -rf ./template/gradle
cp -r ./runtime/gradle ./template/gradle
cp ./runtime/gradle.properties ./runtime/gradlew ./runtime/gradlew.bat ./template

# template_files are common for template and aot_artifacts
TARGET_FILES=$(cat template/template_files.txt)
for TARGET_FILE in $TARGET_FILES; do
    TARGET_DIR=$(dirname $TARGET_FILE)
    mkdir -p $COMMON_OUT_PATH/template/$TARGET_DIR
    cp ./template/$TARGET_FILE $COMMON_OUT_PATH/template/$TARGET_FILE
done

rm -rf ./template/app/src
for TARGET in arm64-v8a x86_64; do
    mkdir -p ./template/app/src/main/jniLibs/$TARGET
    cp ./runtime/runner/src/main/jniLibs/$TARGET/libwebrogue_aot.so ./template/app/src/main/jniLibs/$TARGET/libwebrogue_aot.so
    cp ./runtime/runner/src/main/jniLibs/$TARGET/libwebrogue.so ./template/app/src/main/jniLibs/$TARGET/libwebrogue.so
done
mkdir -p ./template/app/src/main/res
cp -r ./runtime/runner/src/main/res/drawable ./runtime/runner/src/main/res/mipmap-* ./template/app/src/main/res
mkdir -p ./template/app/src/main/assets
cp ./runtime/runner/src/main/assets/aot.swrapp ./template/app/src/main/assets/aot.swrapp

# app/common_files are common for runtime/runner, template and aot_artifacts
TARGET_FILES=$(cat template/app/common_files.txt)
for TARGET_FILE in $TARGET_FILES; do
    TARGET_DIR=$(dirname $TARGET_FILE)
    mkdir -p ./template/app/$TARGET_DIR
    cp ./runtime/runner/$TARGET_FILE ./template/app/$TARGET_FILE
    mkdir -p $COMMON_OUT_PATH/template/app/$TARGET_DIR
    cp ./template/app/$TARGET_FILE $COMMON_OUT_PATH/template/app/$TARGET_FILE
done
echo "*" > $COMMON_OUT_PATH/template/.gitignore

# mkdir -p $COMMON_OUT_PATH/template/app/src/main/java/io/github/webrogue_runtime/common
# cp \
#     runtime/common/src/main/java/io/github/webrogue_runtime/common/WebrogueActivity.java \
#     $COMMON_OUT_PATH/template/app/src/main/java/io/github/webrogue_runtime/common

# mkdir -p $COMMON_OUT_PATH/template/app/src/main/java/io/github/webrogue_runtime/runner
# cp \
#     runtime/runner/src/main/java/io/github/webrogue_runtime/runner/WebrogueRunnerActivity.java \
#     $COMMON_OUT_PATH/template/app/src/main/java/io/github/webrogue_runtime/runner


# mkdir -p $COMMON_OUT_PATH/template/app/src/main/jniLibs/arm64-v8a/
# cp \
#     runtime/runner/build/intermediates/cxx/RelWithDebInfo/*/obj/arm64-v8a/libSDL3.so \
#     $COMMON_OUT_PATH/template/app/src/main/jniLibs/arm64-v8a/libSDL3.so
# mkdir -p $COMMON_OUT_PATH/libs
# cp \
#     runtime/runner/.cxx/RelWithDebInfo/*/arm64-v8a/webrogue_runner_common/CMakeFiles/webrogue.dir/webrogue_runtime.c.o \
#     runtime/runner/src/main/cpp/../rust_target/aarch64-linux-android/aot/libwebrogue_android.a \
#     $ANDROID_SDK_ROOT/ndk/$NDK_VERSION/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/aarch64-linux-android/$ANDROID_API_VERSION/libc.so \
#     $ANDROID_SDK_ROOT/ndk/$NDK_VERSION/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/aarch64-linux-android/$ANDROID_API_VERSION/libdl.so \
#     $ANDROID_SDK_ROOT/ndk/$NDK_VERSION/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/aarch64-linux-android/$ANDROID_API_VERSION/liblog.so \
#     $ANDROID_SDK_ROOT/ndk/$NDK_VERSION/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/aarch64-linux-android/$ANDROID_API_VERSION/crtbegin_so.o \
#     $ANDROID_SDK_ROOT/ndk/$NDK_VERSION/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/aarch64-linux-android/$ANDROID_API_VERSION/crtend_so.o \
#     $COMMON_OUT_PATH/libs

# $ANDROID_NDK_PATH/toolchains/llvm/prebuilt/*/bin/llvm-ar qLs \
#     $COMMON_OUT_PATH/libs/libwebrogue_android.a \
#     runtime/runner/.cxx/RelWithDebInfo/*/arm64-v8a/webrogue_runner_common/libwebrogue_static.a \
#     $ANDROID_SDK_ROOT/ndk/$NDK_VERSION/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/aarch64-linux-android/libc++_static.a \
#     $ANDROID_SDK_ROOT/ndk/$NDK_VERSION/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/aarch64-linux-android/libc++abi.a \
#     $ANDROID_SDK_ROOT/ndk/$NDK_VERSION/toolchains/llvm/prebuilt/linux-x86_64/lib/clang/18/lib/linux/libclang_rt.builtins-aarch64-android.a \
#     $ANDROID_SDK_ROOT/ndk/$NDK_VERSION/toolchains/llvm/prebuilt/linux-x86_64/lib/clang/18/lib/linux/aarch64/libunwind.a

# cp \
#     $ANDROID_SDK_ROOT/ndk/$NDK_VERSION/toolchains/llvm/prebuilt/linux-x86_64/sysroot/NOTICE \
#     $COMMON_OUT_PATH/libs/NOTICE

# cp \
#     runtime/gradlew \
#     runtime/gradlew.bat \
#     $COMMON_OUT_PATH/template

# mkdir -p $COMMON_OUT_PATH/template/gradle/wrapper
# cp \
#     runtime/gradle/wrapper/gradle-wrapper.jar \
#     runtime/gradle/wrapper/gradle-wrapper.properties \
#     $COMMON_OUT_PATH/template/gradle/wrapper

cat /dev/urandom | head -c 32 > $COMMON_OUT_PATH/template_id

for TARGET in arm64-v8a x86_64; do
    if [ "$TARGET" = "arm64-v8a" ]; then
        RUST_TARGET=aarch64
    else
        RUST_TARGET="$TARGET"
    fi
    for VULKAN_TYPE in 'novulkan' 'vulkan'
    do
        OUT_PATH=../aot_artifacts/android-${RUST_TARGET}cpu-$VULKAN_TYPE
        rm -rf $OUT_PATH
        mkdir -p $OUT_PATH
        OUT_PATH="$(realpath $OUT_PATH)"
        cp ./runtime/runner/src/main/jniLibs-$VULKAN_TYPE/$TARGET/libwebrogue.so $OUT_PATH/libwebrogue.so
    done
done
