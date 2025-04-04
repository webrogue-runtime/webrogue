cd $(dirname $0)
set -ex


OUT_DIR="../../aot_artifacts/x86_64-windows-msvc"
rm -rf "$OUT_DIR"
cargo install xwin --locked

XWIN_PATH="$(pwd)/xwin"
test -d "$XWIN_PATH" || xwin --accept-license splat --output "$XWIN_PATH"

cargo xwin build --manifest-path=../../crates/aot-lib/Cargo.toml --target-dir=./target --target=x86_64-pc-windows-msvc --features=gfx-fallback-cmake --profile release-lto

mkdir -p "$OUT_DIR"
cp target/x86_64-pc-windows-msvc/release-lto/webrogue_aot_lib.lib "$OUT_DIR"

for win_type in gui console; do
  clang -target x86_64-pc-win32 -c main.c -o $win_type.obj \
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
    -U__GNUC_STDC_INLINE__ \
    -I$XWIN_PATH/sdk/include/um/ \
    -I$XWIN_PATH/sdk/include/shared/ \
    -I$XWIN_PATH/crt/include \
    -I$XWIN_PATH/sdk/include/ucrt \
    -DWR_WIN_TYPE_$win_type

  mv $win_type.obj "$OUT_DIR/$win_type.obj"
done

cp $XWIN_PATH/crt/lib/x86_64/libcmt.lib "$OUT_DIR/libcmt.lib"
cp $XWIN_PATH/crt/lib/x86_64/oldnames.lib "$OUT_DIR/oldnames.lib"

llvm-lib /out:webrogue_aot_lib.lib \
  /machine:x64 \
  target/x86_64-pc-windows-msvc/release-lto/webrogue_aot_lib.lib \
  $XWIN_PATH/crt/lib/x86_64/libvcruntime.lib \
  $XWIN_PATH/crt/lib/x86_64/libcpmt.lib \
  $XWIN_PATH/sdk/lib/ucrt/x86_64/libucrt.lib \
  $XWIN_PATH/sdk/lib/um/x86_64/ws2_32.lib \
  $XWIN_PATH/sdk/lib/um/x86_64/ntdll.lib \
  $XWIN_PATH/sdk/lib/um/x86_64/advapi32.lib \
  $XWIN_PATH/sdk/lib/um/x86_64/bcrypt.lib \
  $XWIN_PATH/sdk/lib/um/x86_64/kernel32.lib \
  $XWIN_PATH/sdk/lib/um/x86_64/userenv.lib \
  $XWIN_PATH/sdk/lib/um/x86_64/oleaut32.lib \
  $XWIN_PATH/sdk/lib/um/x86_64/ole32.lib \
  $XWIN_PATH/sdk/lib/um/x86_64/gdi32.lib \
  $XWIN_PATH/sdk/lib/um/x86_64/user32.lib \
  $XWIN_PATH/sdk/lib/um/x86_64/imm32.lib \
  $XWIN_PATH/sdk/lib/um/x86_64/version.lib \
  $XWIN_PATH/sdk/lib/um/x86_64/setupapi.lib \
  $XWIN_PATH/sdk/lib/um/x86_64/winmm.lib \
  $XWIN_PATH/sdk/lib/um/x86_64/shell32.lib \
  $XWIN_PATH/sdk/lib/um/x86_64/uuid.lib

mv webrogue_aot_lib.lib "$OUT_DIR/webrogue_aot_lib.lib"


sh ../get_angle.sh
cp ../libEGL.dll "$OUT_DIR/libEGL.dll"
cp ../libGLESv2.dll "$OUT_DIR/libGLESv2.dll"

cargo run --manifest-path=../../crates/aot-compiler/Cargo.toml --target-dir=../../target --release object ../../examples/raylib/raylib.wrapp aot.obj x86_64-windows-msvc
lld-link \
    "-out:aot.exe" \
    "-nologo" \
    "-machine:x64" \
    "aot.obj" \
    "../../aot_artifacts/x86_64-windows-msvc/console.obj" \
    "../../aot_artifacts/x86_64-windows-msvc/webrogue_aot_lib.lib" \
    "../../aot_artifacts/x86_64-windows-msvc/oldnames.lib" \
    "../../aot_artifacts/x86_64-windows-msvc/libcmt.lib" \
    /nodefaultlib \
    /threads:1
rm aot.obj
