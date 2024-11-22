cd $(dirname $0)
set -ex

OUT_DIR="../aot_artifacts/x86_64-windows-gnu"
rm -rf "$OUT_DIR"

MINGW_DIR_NAME=llvm-mingw-20241119-ucrt-ubuntu-20.04-x86_64

test -f $MINGW_DIR_NAME.tar.xz || wget https://github.com/mstorsjo/llvm-mingw/releases/download/20241119/$MINGW_DIR_NAME.tar.xz
test -d $MINGW_DIR_NAME || tar xf $MINGW_DIR_NAME.tar.xz

export PATH="$(pwd)/$MINGW_DIR_NAME/bin:$PATH"

export CMAKE_TOOLCHAIN_FILE="$(pwd)/mingw_llvm_toolchain.cmake"

cargo build \
    -vv \
    --manifest-path=../crates/aot-lib/Cargo.toml \
    --target-dir=./target \
    --target=x86_64-pc-windows-gnullvm \
    --profile release-lto \
    --features=gfx-fallback-cc

# rm -rf mingw_sdl_build
cmake -S ../crates/gfx-fallback/SDL -B mingw_sdl_build -DCMAKE_BUILD_TYPE=Release --toolchain=$(pwd)/mingw_llvm_toolchain.cmake
cmake --build mingw_sdl_build --target SDL2-static

./$MINGW_DIR_NAME/bin/x86_64-w64-mingw32-clang main.c -c -o main.o

rm -f main.exe
rm -f process_dump*
# strace -s 1000 -o process_dump -ff ./$MINGW_DIR_NAME/bin/x86_64-w64-mingw32-clang \
#     main.obj \
#     target/x86_64-pc-windows-gnullvm/release-lto/libwebrogue_aot_lib.a \
#     wr_aot.obj \
#     mingw_sdl_build/libSDL2.a \
#     -lbcrypt \
#     -lws2_32 \
#     -lntoskrnl \
#     -limm32 \
#     -lgdi32 \
#     -lwinmm \
#     -lole32 \
#     -lcfgmgr32 \
#     -loleaut32 \
#     -lversion \
#     -lsetupapi \
#     -lntdll \
#     -o main.exe

mkdir -p "$OUT_DIR"
cp target/x86_64-pc-windows-gnullvm/release-lto/libwebrogue_aot_lib.a "$OUT_DIR"

llvm-ar qLs \
    "$OUT_DIR/libwebrogue_aot_lib.a" \
    $MINGW_DIR_NAME/x86_64-w64-mingw32/lib/libbcrypt.a \
    $MINGW_DIR_NAME/x86_64-w64-mingw32/lib/libws2_32.a \
    $MINGW_DIR_NAME/x86_64-w64-mingw32/lib/libntoskrnl.a \
    $MINGW_DIR_NAME/x86_64-w64-mingw32/lib/libimm32.a \
    $MINGW_DIR_NAME/x86_64-w64-mingw32/lib/libgdi32.a \
    $MINGW_DIR_NAME/x86_64-w64-mingw32/lib/libwinmm.a \
    $MINGW_DIR_NAME/x86_64-w64-mingw32/lib/libole32.a \
    $MINGW_DIR_NAME/x86_64-w64-mingw32/lib/libcfgmgr32.a \
    $MINGW_DIR_NAME/x86_64-w64-mingw32/lib/liboleaut32.a \
    $MINGW_DIR_NAME/x86_64-w64-mingw32/lib/libversion.a \
    $MINGW_DIR_NAME/x86_64-w64-mingw32/lib/libsetupapi.a \
    $MINGW_DIR_NAME/x86_64-w64-mingw32/lib/libntdll.a \
    $MINGW_DIR_NAME/x86_64-w64-mingw32/lib/libadvapi32.a \
    $MINGW_DIR_NAME/x86_64-w64-mingw32/lib/libshell32.a \
    $MINGW_DIR_NAME/x86_64-w64-mingw32/lib/libuser32.a \
    $MINGW_DIR_NAME/x86_64-w64-mingw32/lib/libmingw32.a \
    $MINGW_DIR_NAME/x86_64-w64-mingw32/lib/libmoldname.a \
    $MINGW_DIR_NAME/x86_64-w64-mingw32/lib/libunwind.a \
    $MINGW_DIR_NAME/x86_64-w64-mingw32/lib/libmingwex.a \
    $MINGW_DIR_NAME/x86_64-w64-mingw32/lib/libmsvcrt.a \
    $MINGW_DIR_NAME/x86_64-w64-mingw32/lib/libkernel32.a \
    $MINGW_DIR_NAME/lib/clang/19/lib/windows/libclang_rt.builtins-x86_64.a \
    mingw_sdl_build/libSDL2.a

mv main.o "$OUT_DIR"
cp \
    $MINGW_DIR_NAME/x86_64-w64-mingw32/lib/crt2.o \
    $MINGW_DIR_NAME/x86_64-w64-mingw32/lib/crtbegin.o \
    $MINGW_DIR_NAME/x86_64-w64-mingw32/lib/crtend.o \
    "$OUT_DIR"

test -f windows_x64.zip || wget https://github.com/webrogue-runtime/angle-builder/releases/latest/download/windows_x64.zip
test -f libEGL.dll || unzip -j windows_x64.zip x64/libEGL.dll
cp libEGL.dll "$OUT_DIR/libEGL.dll"
test -f libGLESv2.dll || unzip -j windows_x64.zip x64/libGLESv2.dll
cp libGLESv2.dll "$OUT_DIR/libGLESv2.dll"
