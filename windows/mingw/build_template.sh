cd $(dirname $0)
set -ex

OUT_DIR="../../aot_artifacts/x86_64-windows-gnu"
rm -rf "$OUT_DIR"

MINGW_RELEASE=20241119 #20250305
LLVM_MAJOR_VERSION=19

MINGW_DIR_NAME=llvm-mingw-$MINGW_RELEASE-ucrt-ubuntu-20.04-x86_64
test -f $MINGW_DIR_NAME.tar.xz || wget https://github.com/mstorsjo/llvm-mingw/releases/download/$MINGW_RELEASE/$MINGW_DIR_NAME.tar.xz
test -d $MINGW_DIR_NAME || tar xf $MINGW_DIR_NAME.tar.xz

export PATH="$(pwd)/$MINGW_DIR_NAME/bin:$PATH"

export CXXFLAGS_x86_64_pc_windows_gnullvm="-I$(pwd)/$MINGW_DIR_NAME/x86_64-w64-mingw32/include/c++/v1"
cargo build \
    --manifest-path=../../crates/aot-lib/Cargo.toml \
    --target-dir=./target \
    --target=x86_64-pc-windows-gnullvm \
    --features=gfx-fallback-cc \
    --profile=aot

# rm -rf sdl_build
cmake -S ../../crates/gfx-fallback/SDL -B sdl_build -DCMAKE_BUILD_TYPE=Release -DSDL_OPENGL=OFF -DSDL_OPENGLES=ON -DSDL_CMAKE_DEBUG_POSTFIX= --toolchain=$(pwd)/mingw_llvm_toolchain.cmake
cmake --build sdl_build --target SDL2-static

./$MINGW_DIR_NAME/bin/x86_64-w64-mingw32-clang -g -Og main.c -c -o main.o

rm -f main.exe
rm -f process_dump*
# strace -s 1000 -o process_dump -ff ./$MINGW_DIR_NAME/bin/x86_64-w64-mingw32-clang \
#     main.obj \
#     target/x86_64-pc-windows-gnullvm/aot/libwebrogue_aot_lib.a \
#     wr_aot.obj \
#     sdl_build/libSDL2.a \
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
cp target/x86_64-pc-windows-gnullvm/aot/libwebrogue_aot_lib.a "$OUT_DIR"

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
    $MINGW_DIR_NAME/x86_64-w64-mingw32/lib/libuserenv.a \
    $MINGW_DIR_NAME/lib/clang/$LLVM_MAJOR_VERSION/lib/windows/libclang_rt.builtins-x86_64.a \
    $MINGW_DIR_NAME/x86_64-w64-mingw32/lib/libc++.a \
    $MINGW_DIR_NAME/x86_64-w64-mingw32/lib/libc++abi.a \
    sdl_build/libSDL2.a

mv main.o "$OUT_DIR"
cp \
    $MINGW_DIR_NAME/x86_64-w64-mingw32/lib/crt2.o \
    $MINGW_DIR_NAME/x86_64-w64-mingw32/lib/crtbegin.o \
    $MINGW_DIR_NAME/x86_64-w64-mingw32/lib/crtend.o \
    "$OUT_DIR"

sh ../get_angle.sh
cp ../libEGL.dll "$OUT_DIR/libEGL.dll"
cp ../libGLESv2.dll "$OUT_DIR/libGLESv2.dll"
