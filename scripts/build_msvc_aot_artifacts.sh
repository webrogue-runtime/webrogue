cd $(dirname $0)
set -ex


OUT_DIR="../aot_artifacts/x86_64-windows-msvc"
rm -rf "$OUT_DIR"
cargo install xwin --locked

XWIN_PATH="$(pwd)/xwin"
test -d "$XWIN_PATH" || xwin --accept-license splat --output "$XWIN_PATH"

cargo xwin build --manifest-path=../crates/aot-lib/Cargo.toml --target-dir=./target --target=x86_64-pc-windows-msvc --profile release-lto --features=gfx-fallback-cc

mkdir -p "$OUT_DIR"
cp target/x86_64-pc-windows-msvc/release-lto/webrogue_aot_lib.lib "$OUT_DIR"

clang -target x86_64-pc-win32 -c main.c -o main.obj \
  -fms-compatibility-version=19 \
  -fms-extensions \
  -fdelayed-template-parsing \
  -fexceptions \
  -mthread-model posix \
  -fno-threadsafe-statics \
  -Wno-msvc-not-found \
  -DWIN32 \
  -D_WIN32 \
  -D_MT \
  -D_DLL \
  -Xclang -disable-llvm-verifier \
  -D_CRT_SECURE_NO_WARNINGS \
  -D_CRT_NONSTDC_NO_DEPRECATE \
  -U__GNUC__ \
  -U__gnu_linux__ \
  -U__GNUC_MINOR__ \
  -U__GNUC_PATCHLEVEL__ \
  -U__GNUC_STDC_INLINE__  

mv main.obj "$OUT_DIR/main.obj"

cp $XWIN_PATH/crt/lib/x86_64/msvcrt.lib "$OUT_DIR/msvcrt.lib"
cp $XWIN_PATH/sdk/lib/ucrt/x86_64/ucrt.lib "$OUT_DIR/ucrt.lib"
cp $XWIN_PATH/crt/lib/x86_64/oldnames.lib "$OUT_DIR/oldnames.lib"
cp $XWIN_PATH/crt/lib/x86_64/vcruntime.lib "$OUT_DIR/vcruntime.lib"
cp $XWIN_PATH/sdk/lib/um/x86_64/ws2_32.lib "$OUT_DIR/ws2_32.lib"
cp $XWIN_PATH/sdk/lib/um/x86_64/ws2_32.lib "$OUT_DIR/ws2_32.lib"
cp $XWIN_PATH/sdk/lib/um/x86_64/ntdll.lib "$OUT_DIR/ntdll.lib"
cp $XWIN_PATH/sdk/lib/um/x86_64/advapi32.lib "$OUT_DIR/advapi32.lib"
cp $XWIN_PATH/sdk/lib/um/x86_64/bcrypt.lib "$OUT_DIR/bcrypt.lib"
cp $XWIN_PATH/sdk/lib/um/x86_64/kernel32.lib "$OUT_DIR/kernel32.lib"

test -f SDL2-devel-2.30.9-VC.zip || wget https://github.com/libsdl-org/SDL/releases/download/release-2.30.9/SDL2-devel-2.30.9-VC.zip
test -f SDL2.lib || unzip -j SDL2-devel-2.30.9-VC.zip SDL2-2.30.9/lib/x64/SDL2.lib
mv SDL2.lib "$OUT_DIR/SDL2.lib"
test -f SDL2.dll || unzip -j SDL2-devel-2.30.9-VC.zip SDL2-2.30.9/lib/x64/SDL2.dll
cp SDL2.dll "$OUT_DIR/SDL2.dll"

test -f windows_x64.zip || wget https://github.com/webrogue-runtime/angle-builder/releases/latest/download/windows_x64.zip
test -f libEGL.dll || unzip -j windows_x64.zip x64/libEGL.dll
cp libEGL.dll "$OUT_DIR/libEGL.dll"
test -f libGLESv2.dll || unzip -j windows_x64.zip x64/libGLESv2.dll
cp libGLESv2.dll "$OUT_DIR/libGLESv2.dll"
