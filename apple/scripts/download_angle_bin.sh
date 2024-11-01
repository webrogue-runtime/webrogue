set -ex

if [ ! -f "macos_fat.zip" ]; then
    wget https://github.com/webrogue-runtime/angle-builder/releases/latest/download/macos_fat.zip -O macos_fat.zip
fi
if [ ! -f "libEGL.dylib" ]; then
    unzip -j macos_fat.zip fat/libEGL.dylib
fi
if [ ! -f "libGLESv2.dylib" ]; then
    unzip -j macos_fat.zip fat/libGLESv2.dylib
fi
