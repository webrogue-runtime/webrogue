set -ex

if [ ! -f "libEGL.dylib" ]; then
    cp $SRCROOT/bin/libEGL.macosx.dylib libEGL.dylib 
fi
if [ ! -f "libGLESv2.dylib" ]; then
    cp $SRCROOT/bin/libGLESv2.macosx.dylib libGLESv2.dylib
fi
cp $SRCROOT/bin/libwebrogue_macos.macosx.a libwebrogue_macos.a
cp $SRCROOT/bin/libSDL2.macosx.a libSDL2.a
cp $SRCROOT/bin/libGFXStream.macosx.a libGFXStream.a
