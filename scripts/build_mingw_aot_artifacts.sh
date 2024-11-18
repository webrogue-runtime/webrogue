cd $(dirname $0)
set -ex

rm -rf ../aot_artifacts
mkdir ../aot_artifacts

export NUM_JOBS=$(nproc)

# apt install mingw-w64
# rustup target add x86_64-unknown-linux-gnu
cargo build --manifest-path=../crates/aot-lib/Cargo.toml --target-dir=./target --target=x86_64-pc-windows-gnu --features=gfx-fallback --profile release-lto

mkdir ../aot_artifacts/x86_64-windows-gnu
cp target/x86_64-pc-windows-gnu/release-lto/libwebrogue_aot_lib.a ../aot_artifacts/x86_64-windows-gnu

x86_64-w64-mingw32-gcc main.c -nostdlib -c -o main.o

cargo run \
    --manifest-path=../crates/aot-compiler/Cargo.toml \
    --target-dir=./target \
    --release \
    ../examples/gears/gears.webc \
    wr_aot.o \
    x86_64-linux-gnu

rm -f process_dump*
strace -v -s 1000 -o process_dump -ff x86_64-w64-mingw32-gcc \
    main.o \
    ../aot_artifacts/x86_64-windows-gnu/libwebrogue_aot_lib.a \
    /usr/x86_64-w64-mingw32/lib/libkernel32.a \
    /usr/x86_64-w64-mingw32/lib/libuser32.a \
    /usr/x86_64-w64-mingw32/lib/libgdi32.a \
    /usr/x86_64-w64-mingw32/lib/libwinmm.a \
    /usr/x86_64-w64-mingw32/lib/libimm32.a \
    /usr/x86_64-w64-mingw32/lib/libole32.a \
    /usr/x86_64-w64-mingw32/lib/liboleaut32.a \
    /usr/x86_64-w64-mingw32/lib/libversion.a \
    /usr/x86_64-w64-mingw32/lib/libuuid.a \
    /usr/x86_64-w64-mingw32/lib/libadvapi32.a \
    /usr/x86_64-w64-mingw32/lib/libsetupapi.a \
    /usr/x86_64-w64-mingw32/lib/libshell32.a \
    /usr/x86_64-w64-mingw32/lib/libws2_32.a \
    /usr/x86_64-w64-mingw32/lib/libntdll.a \
    /usr/x86_64-w64-mingw32/lib/libbcrypt.a \
    wr_aot.o \
    -o a.exe \
    2>e

          

# clang \
#     main.o \
#     ../aot_artifacts/x86_64-linux-gnu/libwebrogue_aot_lib.a \
#     wr_aot.o \
#     -lm \
#     -o a2.out \
#     -fuse-ld=lld \
#     -Wl,--trace \
# | grep -v "wr_aot.o" \
# | sed 's/(.\+)//g' \
# | uniq \
# > trace

# llvm-ar q ../aot_artifacts/x86_64-linux-gnu/libwebrogue_aot_lib.a \
#     "/lib/x86_64-linux-gnu/Scrt1.o" \
#     "/lib/x86_64-linux-gnu/crti.o" \
#     "/usr/bin/../lib/gcc/x86_64-linux-gnu/13/crtbeginS.o" \
#     "main.o" \
#     "/lib/x86_64-linux-gnu/crtn.o"

# llvm-ar qLs ../aot_artifacts/x86_64-linux-gnu/libwebrogue_aot_lib.a \
#     /usr/lib/x86_64-linux-gnu/libc_nonshared.a

# cp /lib/x86_64-linux-gnu/libm.so.6 ../aot_artifacts/x86_64-linux-gnu/
# cp /lib/x86_64-linux-gnu/libgcc_s.so.1 ../aot_artifacts/x86_64-linux-gnu/
# cp /lib/x86_64-linux-gnu/libc.so.6 ../aot_artifacts/x86_64-linux-gnu/
# cp /usr/bin/../lib/gcc/x86_64-linux-gnu/13/crtendS.o ../aot_artifacts/x86_64-linux-gnu/
# rm main.o
