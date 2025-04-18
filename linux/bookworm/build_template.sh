cd $(dirname $(dirname $0))
set -ex

OUT_DIR="../aot_artifacts/x86_64-linux-gnu"
rm -rf "$OUT_DIR"

export NUM_JOBS=$(nproc)

# rustup target add x86_64-unknown-linux-gnu
cargo build --manifest-path=../crates/aot-lib/Cargo.toml --target-dir=./target --target=x86_64-unknown-linux-gnu --features=gfx-fallback-cmake --profile aot

mkdir -p "$OUT_DIR"
cp target/x86_64-unknown-linux-gnu/aot/libwebrogue_aot_lib.a "$OUT_DIR"

clang main.c -nostdlib -c -o main.o


rm -f process_dump*
# strace -s 1000 -o process_dump -ff clang++ -static -lc++abi \
#     main.o \
#     ../aot_artifacts/x86_64-linux-gnu/libwebrogue_aot_lib.a \
#     empty.gnu.o \
#     -lm \
#     -lpthread \
#     -ldl \
#     -o a.out \
#     -fuse-ld=lld \
#     -Wl,--threads=1

llvm-ar q \
    "$OUT_DIR/libwebrogue_aot_lib.a" \
    "main.o"
    

llvm-ar qLs \
    "$OUT_DIR/libwebrogue_aot_lib.a" \
    /usr/lib/x86_64-linux-gnu/libc_nonshared.a \
    /usr/lib/gcc/x86_64-linux-gnu/$GCC_VERSION/libstdc++.a 

cp \
    "/usr/lib/gcc/x86_64-linux-gnu/$GCC_VERSION/crtbeginS.o" \
    "/usr/lib/x86_64-linux-gnu/Scrt1.o" \
    "/usr/lib/x86_64-linux-gnu/crti.o" \
    /lib/x86_64-linux-gnu/libm.so.6 \
    /lib/x86_64-linux-gnu/libgcc_s.so.1 \
    /usr/lib/gcc/x86_64-linux-gnu/$GCC_VERSION/crtendS.o \
    /lib/x86_64-linux-gnu/libdl.so.2 \
    /lib/x86_64-linux-gnu/libpthread.so.0 \
    /lib/x86_64-linux-gnu/libc.so.6 \
    "/usr/lib/x86_64-linux-gnu/crtn.o" \
    "$OUT_DIR"

strip --strip-debug $OUT_DIR/*

rm main.o

ld.lld \
    -pie \
    --no-dependent-libraries \
    --hash-style=gnu \
    --build-id \
    --eh-frame-hdr \
    -m \
    elf_x86_64 \
    --strip-all \
    --gc-sections \
    -dynamic-linker \
    /lib64/ld-linux-x86-64.so.2 \
    -z \
    relro \
    -o \
    aot \
    --no-as-needed \
    "$OUT_DIR/Scrt1.o" \
    --no-as-needed \
    "$OUT_DIR/crti.o" \
    --no-as-needed \
    "$OUT_DIR/crtbeginS.o" \
    "$OUT_DIR/libwebrogue_aot_lib.a" \
    empty.gnu.o \
    "$OUT_DIR/libm.so.6" \
    --as-needed \
    "$OUT_DIR/libc.so.6" \
    "$OUT_DIR/libgcc_s.so.1" \
    "$OUT_DIR/libdl.so.2" \
    "$OUT_DIR/libpthread.so.0" \
    --no-as-needed \
    "$OUT_DIR/crtendS.o" \
    --no-as-needed \
    "$OUT_DIR/crtn.o"

rm aot
