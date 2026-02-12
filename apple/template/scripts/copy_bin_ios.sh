set -ex

if [ -f "$SRCROOT/bin/$PLATFORM_NAME" ]; then
    echo "error: $(cat $SRCROOT/bin/$PLATFORM_NAME)" >/dev/stderr
    exit 1
fi

cp $SRCROOT/bin/$PLATFORM_NAME/libwebrogue_ios.a libwebrogue_ios.a
cp $SRCROOT/bin/$PLATFORM_NAME/libGFXStream.a libGFXStream.a
cp $SRCROOT/bin/$PLATFORM_NAME/librunnerlib.a librunnerlib.a
