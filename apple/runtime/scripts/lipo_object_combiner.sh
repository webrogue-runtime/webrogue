set -ex

INPUT_DIR=$1
OUTPUT_PATH=$2

LIPO_PATHS=()
LIPO_PATHS_I=0

for DEST_ARCH in $ARCHS; do
    LIPO_PATHS[$LIPO_PATHS_I]="$INPUT_DIR/aot_$DEST_ARCH.o"
    LIPO_PATHS_I=$(expr $LIPO_PATHS_I '+' 1)
done

lipo -create "${LIPO_PATHS[@]}" -output "$OUTPUT_PATH"

# /Users/artem/Library/Developer/Xcode/DerivedData/webrogue-abzsnfivuvgrlwgdtioifbakhiax/Build/Products/rust_artifacts/runtime/Debug/macosx/aot_lipo.o
# /Users/artem/Library/Developer/Xcode/DerivedData/webrogue-abzsnfivuvgrlwgdtioifbakhiax/Build/Products/rust_artifacts/runner/Debug/macosx/aot_lipo.o
