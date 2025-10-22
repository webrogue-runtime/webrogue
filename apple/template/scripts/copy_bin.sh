set -ex

if [ ! -f "libMoltenVK.dylib" ]; then
    cp $SRCROOT/bin/macos/libMoltenVK.dylib libMoltenVK.dylib 
fi
cp $SRCROOT/bin/macos/libwebrogue_macos.a libwebrogue_macos.a
cp $SRCROOT/bin/macos/libGFXStream.a libGFXStream.a
