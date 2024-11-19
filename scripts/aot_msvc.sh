cd $(dirname $0)
set -ex

XWIN_PATH="$(pwd)/xwin"

sh build_msvc_aot_artifacts.sh

cargo run \
    --manifest-path=../crates/aot-compiler/Cargo.toml \
    --target-dir=./target \
    --release \
    ../examples/raylib/raylib.webc \
    wr_aot.obj \
    x86_64-windows-msvc

# strace -s 100000 -o process_dump -ff  clang \
#     -fuse-ld=lld \
#     -target x86_64-pc-win32 \
#     -Wl,-machine:x64 \
#     -fmsc-version=1900 \
#     -o hello.exe \
#     main.obj \
#     ../aot_artifacts/x86_64-windows-msvc/webrogue_aot_lib.lib \
#     wr_aot.obj \
#     ../aot_artifacts/x86_64-windows-msvc/SDL2.lib \
#     ../aot_artifacts/x86_64-windows-msvc/ws2_32.lib \
#     ../aot_artifacts/x86_64-windows-msvc/ntdll.lib \
#     ../aot_artifacts/x86_64-windows-msvc/advapi32.lib \
#     ../aot_artifacts/x86_64-windows-msvc/bcrypt.lib \
#     -L../aot_artifacts/x86_64-windows-msvc/ \
#     -nostdlib ../aot_artifacts/x86_64-windows-msvc/msvcrt.lib \
#     -Wno-msvc-not-found 

rm -f a.exe

lld-link \
    "-out:a.exe" \
    "-libpath:../aot_artifacts/x86_64-windows-msvc/" \
    "-nologo" \
    "-machine:x64" \
    "main.obj" \
    "../aot_artifacts/x86_64-windows-msvc/webrogue_aot_lib.lib" \
    "wr_aot.obj" \
    "../aot_artifacts/x86_64-windows-msvc/SDL2.lib" \
    "../aot_artifacts/x86_64-windows-msvc/ws2_32.lib" \
    "../aot_artifacts/x86_64-windows-msvc/ntdll.lib" \
    "../aot_artifacts/x86_64-windows-msvc/advapi32.lib" \
    "../aot_artifacts/x86_64-windows-msvc/bcrypt.lib" \
    "../aot_artifacts/x86_64-windows-msvc/msvcrt.lib" \
    "../aot_artifacts/x86_64-windows-msvc/kernel32.lib" \
    "../aot_artifacts/x86_64-windows-msvc/oldnames.lib" \
    "../aot_artifacts/x86_64-windows-msvc/ucrt.lib" \
    "../aot_artifacts/x86_64-windows-msvc/vcruntime.lib" \
    /nodefaultlib \
    /threads:1

rm -f wr_aot.obj
