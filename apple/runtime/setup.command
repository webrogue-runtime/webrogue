cd $(dirname $0)
set -ex

sh scripts/download_angle_ios_headers.sh
sh scripts/download_angle_ios_xcframeworks.sh
sh scripts/download_sdl_src.sh

WRAPP_PATH=../../examples/empty/empty.wrapp

if test -f $WRAPP_PATH
then
    cp $WRAPP_PATH macos/runner/aot.swrapp
    cp $WRAPP_PATH ios/runner/aot.swrapp

    COMPILE_COMMAND="cargo run --release --no-default-features --features=compile compile object"
    $COMPILE_COMMAND $WRAPP_PATH macos/runner/aot.x86_64.macosx.o x86_64-apple-darwin --pic
    $COMPILE_COMMAND $WRAPP_PATH macos/runner/aot.arm64.macosx.o arm64-apple-darwin --pic
    $COMPILE_COMMAND $WRAPP_PATH ios/runner/aot.x86_64.iphonesimulator.o x86_64-apple-ios --pic
    $COMPILE_COMMAND $WRAPP_PATH ios/runner/aot.arm64.iphonesimulator.o arm64-apple-ios-sim --pic
    $COMPILE_COMMAND $WRAPP_PATH ios/runner/aot.arm64.iphoneos.o arm64-apple-ios --pic
fi

xcodegen -c
