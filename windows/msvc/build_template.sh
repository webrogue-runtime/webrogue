cd $(dirname $0)
set -ex

OUT_DIR="../../aot_artifacts/x86_64-windows-msvc"
rm -rf "$OUT_DIR"

XWIN_PATH="$(pwd)/xwin"
test -d "$XWIN_PATH" || xwin --accept-license splat --output "$XWIN_PATH"

cargo xwin build --manifest-path=../../crates/aot-lib/Cargo.toml --target-dir=./target --target=x86_64-pc-windows-msvc --features=gfx-fallback-cmake --profile aot

mkdir -p "$OUT_DIR"

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


cp target/x86_64-pc-windows-msvc/aot/webrogue_aot_lib.lib webrogue_aot_lib.lib
llvm-ar qLs webrogue_aot_lib.lib \
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
  $XWIN_PATH/sdk/lib/um/x86_64/uuid.lib \
  $XWIN_PATH/crt/lib/x86_64/libvcruntime.lib \
  $XWIN_PATH/crt/lib/x86_64/oldnames.lib \
  $XWIN_PATH/crt/lib/x86_64/libcmt.lib

mv webrogue_aot_lib.lib "$OUT_DIR/webrogue_aot_lib.lib"

sh ../get_angle.sh
cp ../libEGL.dll "$OUT_DIR/libEGL.dll"
cp ../libGLESv2.dll "$OUT_DIR/libGLESv2.dll"

# It is possible to preform tree-shaking only if lld-link tool is installed
if lld-link --version
then
  echo "Using lld-link"
  lld_link()
  {
    E=$@
    sh -c "lld-link $E"
  }
else
  which lld || exit
  echo "Emulating lld-link using lld"
  lld_link()
  {
    E=$@
    bash -c "exec -a lld-link $(which lld) $E"
  }
  export PATH="$(pwd):$PATH"
fi



mv "$OUT_DIR/webrogue_aot_lib.lib" webrogue_aot_lib.lib 

llvm-ar t webrogue_aot_lib.lib > lib_content.txt
test -f ../../examples/empty/empty.wrapp && cargo run --no-default-features --features=compile --target-dir=../../target --release compile object ../../examples/empty/empty.wrapp empty.obj x86_64-windows-msvc

for win_type in gui console; do
  # Collect verbose information to preform tree-shaking of resulting static archives
  lld_link \
      "-out:aot.exe" \
      "-nologo" \
      "-machine:x64" \
      "empty.obj" \
      "../../aot_artifacts/x86_64-windows-msvc/$win_type.obj" \
      "webrogue_aot_lib.lib" \
      /nodefaultlib \
      /threads:1 \
      /verbose 2>lld_output_$win_type.txt || { 
        cat lld_output_$win_type.txt
        exit 1
      }
done

python3 filter.py > filtered.txt

# tree-shaking: remove everything not mentioned in lld-link's verbose output
llvm-ar d webrogue_aot_lib.lib $(cat filtered.txt)

mv webrogue_aot_lib.lib "$OUT_DIR/webrogue_aot_lib.lib"

for win_type in gui console; do
  lld_link \
      "-out:aot.exe" \
      "-nologo" \
      "-machine:x64" \
      "empty.obj" \
      "../../aot_artifacts/x86_64-windows-msvc/$win_type.obj" \
      "../../aot_artifacts/x86_64-windows-msvc/webrogue_aot_lib.lib" \
      /nodefaultlib \
      /threads:1
done

rm aot.exe
