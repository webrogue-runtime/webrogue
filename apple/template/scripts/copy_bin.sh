set -ex

if [ ! -f "libEGL.dylib" ]; then
    cp $SRCROOT/bin/macos/libEGL.dylib libEGL.dylib 
fi
if [ ! -f "libGLESv2.dylib" ]; then
    cp $SRCROOT/bin/macos/libGLESv2.dylib libGLESv2.dylib
fi
cp $SRCROOT/bin/macos/libwebrogue_macos.a libwebrogue_macos.a
cp $SRCROOT/bin/macos/libGFXStream.a libGFXStream.a
cp $SRCROOT/bin/macos/libSDL3.a libSDL3.a
