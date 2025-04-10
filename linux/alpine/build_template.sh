cd $(dirname $(dirname $0))
set -ex

OUT_DIR="../aot_artifacts/x86_64-linux-musl"
rm -rf "$OUT_DIR"

export NUM_JOBS=$(nproc)

# rustup target add x86_64-unknown-linux-musl
cargo build --manifest-path=../crates/aot-lib/Cargo.toml --target-dir=./target --target=x86_64-unknown-linux-musl --features=gfx-fallback-cmake --profile release-lto

mkdir -p "$OUT_DIR"
cp target/x86_64-unknown-linux-musl/release-lto/libwebrogue_aot_lib.a "$OUT_DIR"

clang main.c -nostdlib -c -o main.o


rm -f process_dump*
# strace -s 1000 -o process_dump -ff clang++ -static \
#     main.o \
#     ../aot_artifacts/x86_64-linux-musl/libwebrogue_aot_lib.a \
#     empty.musl.o \
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
    /usr/lib/libm.a \
    /usr/lib/libpthread.a \
    /usr/lib/libdl.a \
    /usr/lib/libstdc++.a \
    /usr/lib/libssp_nonshared.a \
    /usr/lib/gcc/x86_64-alpine-linux-musl/*/libgcc.a \
    /usr/lib/gcc/x86_64-alpine-linux-musl/*/libgcc_eh.a \
    /usr/lib/libc.a

cp \
    /usr/lib/crt1.o \
    /usr/lib/crti.o \
    /usr/lib/gcc/x86_64-alpine-linux-musl/*/crtbeginT.o \
    /usr/lib/gcc/x86_64-alpine-linux-musl/*/crtend.o \
    /usr/lib/crtn.o \
    "$OUT_DIR"

strip --strip-debug $OUT_DIR/*

rm main.o

ld.lld \
    -z \
    now \
    -z \
    relro \
    --hash-style=gnu \
    --build-id \
    --eh-frame-hdr \
    -m \
    elf_x86_64 \
    --strip-all \
    --gc-sections \
    -static \
    -o \
    aot \
    "$OUT_DIR/crt1.o" \
    "$OUT_DIR/crti.o" \
    "$OUT_DIR/crtbeginT.o" \
    --as-needed \
    "$OUT_DIR/libwebrogue_aot_lib.a" \
    empty.musl.o \
    "$OUT_DIR/crtend.o" \
    "$OUT_DIR/crtn.o"

rm aot
