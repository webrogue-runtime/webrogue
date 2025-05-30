set -ex

cd $(dirname $0)/../external

if [ ! -d "libEGL.xcframework" ]; then
    curl -L https://github.com/webrogue-runtime/angle-builder/releases/latest/download/ios_libEGL.xcframework.zip -o libEGL.xcframework.zip
    tar -xf libEGL.xcframework.zip
    rm libEGL.xcframework.zip
fi

if [ ! -d "libGLESv2.xcframework" ]; then
    curl -L https://github.com/webrogue-runtime/angle-builder/releases/latest/download/ios_libGLESv2.xcframework.zip -o libGLESv2.xcframework.zip
    tar -xf libGLESv2.xcframework.zip
    rm libGLESv2.xcframework.zip
fi

