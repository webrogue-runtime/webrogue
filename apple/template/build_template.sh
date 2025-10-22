# apple/template
cd $(dirname $0)
set -e
# apple/runtime
cd ../runtime

sh setup.command

XC_FLAGS="-destination generic/platform=macOS -workspace webrogue.xcworkspace -scheme MacOS_Runner_ReleaseLocal -configuration ReleaseLocal"
XC_BUILD_DIR=$(xcodebuild $XC_FLAGS -showBuildSettings | grep -m 1 "BUILD_DIR =" | grep -oEi "\/.*" || exit 3)
xcodebuild $XC_FLAGS -parallelizeTargets -allowProvisioningUpdates

rm -rf ../template/bin
mkdir -p ../template/bin
mkdir -p ../template/bin/macos
cp $XC_BUILD_DIR/rust_artifacts/runner/ReleaseLocal/macosx/libwebrogue_macos.a ../template/bin/macos/libwebrogue_macos.a
cp $XC_BUILD_DIR/ReleaseLocal/libGFXStream.a ../template/bin/macos/libGFXStream.a
cp external/MoltenVK/MoltenVK/dylib/macOS/libMoltenVK.dylib ../template/bin/macos/libMoltenVK.dylib

# XC_FLAGS="-workspace webrogue.xcworkspace -scheme iOS_Runner_ReleaseLocal -configuration ReleaseLocal -destination"
# XC_DESTINATION_FLAG="generic/platform=iOS Simulator"
# XC_BUILD_DIR=$(xcodebuild $XC_FLAGS "$XC_DESTINATION_FLAG" -showBuildSettings | grep -m 1 "BUILD_DIR =" | grep -oEi "\/.*" || exit 3)
# xcodebuild $XC_FLAGS "$XC_DESTINATION_FLAG" -parallelizeTargets -allowProvisioningUpdates


for IOS_ENV in iphoneos iphonesimulator
do
    mkdir -p ../template/bin/$IOS_ENV
    case "$IOS_ENV" in
        iphonesimulator)
            XC_DESTINATION_FLAG="generic/platform=iOS Simulator"
            ;;
        iphoneos)
            XC_DESTINATION_FLAG="generic/platform=iOS"
            ;;
        *)
            exit 1
            ;;
    esac
    XC_FLAGS="-workspace webrogue.xcworkspace -scheme Cargo_iOS_runner -configuration ReleaseLocal -destination"
    XC_BUILD_DIR=$(xcodebuild $XC_FLAGS "$XC_DESTINATION_FLAG" -showBuildSettings | grep -m 1 "BUILD_DIR =" | grep -oEi "\/.*" || exit 3)
    xcodebuild $XC_FLAGS "$XC_DESTINATION_FLAG" -parallelizeTargets -allowProvisioningUpdates
    cp $XC_BUILD_DIR/rust_artifacts/ios_runner/ReleaseLocal/$IOS_ENV/libwebrogue_ios.a ../template/bin/$IOS_ENV/libwebrogue_ios.a

    XC_FLAGS="-workspace webrogue.xcworkspace -scheme runnerlib -configuration ReleaseLocal -destination"
    XC_BUILD_DIR=$(xcodebuild $XC_FLAGS "$XC_DESTINATION_FLAG" -showBuildSettings | grep -m 1 "BUILD_DIR =" | grep -oEi "\/.*" || exit 3)
    xcodebuild $XC_FLAGS "$XC_DESTINATION_FLAG" -parallelizeTargets -allowProvisioningUpdates
    cp $XC_BUILD_DIR/ReleaseLocal-$IOS_ENV/librunnerlib.a ../template/bin/$IOS_ENV/librunnerlib.a

    XC_FLAGS="-workspace webrogue.xcworkspace -scheme GFXStream_iOS -configuration ReleaseLocal -destination"
    XC_BUILD_DIR=$(xcodebuild $XC_FLAGS "$XC_DESTINATION_FLAG" -showBuildSettings | grep -m 1 "BUILD_DIR =" | grep -oEi "\/.*" || exit 3)
    xcodebuild $XC_FLAGS "$XC_DESTINATION_FLAG" -parallelizeTargets -allowProvisioningUpdates
    cp $XC_BUILD_DIR/ReleaseLocal-$IOS_ENV/libGFXStream.a ../template/bin/$IOS_ENV/libGFXStream.a
done

# apple
cd ..
rm -rf template/aot
mkdir -p template/aot
cp runtime/macos/runner/aot.arm64.macosx.o template/aot/aot.arm64.macosx.o
cp runtime/macos/runner/aot.x86_64.macosx.o template/aot/aot.x86_64.macosx.o
cp runtime/ios/runner/aot.x86_64.iphonesimulator.o template/aot/aot.x86_64.iphonesimulator.o
cp runtime/ios/runner/aot.arm64.iphonesimulator.o template/aot/aot.arm64.iphonesimulator.o
cp runtime/ios/runner/aot.arm64.iphoneos.o template/aot/aot.arm64.iphoneos.o
cp runtime/macos/runner/aot.swrapp template/aot.swrapp

cp runtime/scripts/lipo_object_combiner.sh template/scripts/lipo_object_combiner.sh

mkdir -p template/macos
cp runtime/macos/runner/main.m template/macos/main.m
cp runtime/macos/runner/runner.entitlements template/macos/runner.entitlements

mkdir -p template/ios
cp runtime/ios/runner/main.m template/ios/main.m
cp runtime/ios/runner/Info.plist template/ios/Info.plist
cp runtime/ios/runner/ios.entitlements template/ios/ios.entitlements
cp -r runtime/external/MoltenVK/MoltenVK/static/MoltenVK.xcframework template/bin/MoltenVK.xcframework
# apple/template
cd template
xcodegen

rm -rf ../../aot_artifacts/apple_xcode
mkdir -p ../../aot_artifacts/apple_xcode
mkdir -p ../../aot_artifacts/apple_xcode/template

TARGET_FILES=$(cat template_files.txt)

for TARGET_FILE in $TARGET_FILES; do
    TARGET_DIR=$(dirname $TARGET_FILE)
    mkdir -p ../../aot_artifacts/apple_xcode/template/$TARGET_DIR
    cp ../../apple/template/$TARGET_FILE ../../aot_artifacts/apple_xcode/template/$TARGET_FILE
done
echo "*" > ../../aot_artifacts/apple_xcode/template/.gitignore

cat /dev/urandom | head -c 32 > ../../aot_artifacts/apple_xcode/template_id
