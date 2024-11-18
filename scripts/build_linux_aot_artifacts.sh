cd $(dirname $0)
set -ex

rm -rf ../aot_artifacts
mkdir ../aot_artifacts

export NUM_JOBS=$(nproc)

# rustup target add x86_64-unknown-linux-gnu
cargo build --manifest-path=../crates/aot-lib/Cargo.toml --target-dir=./target --target=x86_64-unknown-linux-gnu --features=gfx-fallback --profile release-lto

mkdir ../aot_artifacts/x86_64-linux-gnu
cp target/x86_64-unknown-linux-gnu/release-lto/libwebrogue_aot_lib.a ../aot_artifacts/x86_64-linux-gnu

clang main.c -nostdlib -c -o main.o

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

llvm-ar q ../aot_artifacts/x86_64-linux-gnu/libwebrogue_aot_lib.a \
    "/lib/x86_64-linux-gnu/Scrt1.o" \
    "/lib/x86_64-linux-gnu/crti.o" \
    "/usr/bin/../lib/gcc/x86_64-linux-gnu/13/crtbeginS.o" \
    "main.o" \
    "/lib/x86_64-linux-gnu/crtn.o"

llvm-ar qLs ../aot_artifacts/x86_64-linux-gnu/libwebrogue_aot_lib.a \
    /usr/lib/x86_64-linux-gnu/libc_nonshared.a

cp /lib/x86_64-linux-gnu/libm.so.6 ../aot_artifacts/x86_64-linux-gnu/
cp /lib/x86_64-linux-gnu/libgcc_s.so.1 ../aot_artifacts/x86_64-linux-gnu/
cp /lib/x86_64-linux-gnu/libc.so.6 ../aot_artifacts/x86_64-linux-gnu/
cp /usr/bin/../lib/gcc/x86_64-linux-gnu/13/crtendS.o ../aot_artifacts/x86_64-linux-gnu/
rm main.o
