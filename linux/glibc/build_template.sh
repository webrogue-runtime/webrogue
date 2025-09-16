cd $(dirname $(dirname $0))
set -ex

OUT_DIR="../aot_artifacts/x86_64-linux-gnu"
rm -rf "$OUT_DIR"

export NUM_JOBS=$(nproc)

# rustup target add x86_64-unknown-linux-gnu
cargo build --manifest-path=../crates/aot-lib/Cargo.toml --target-dir=./target --target=x86_64-unknown-linux-gnu --features=build-sdl,build-gfxstream-cc --profile aot

mkdir -p "$OUT_DIR"
cp target/x86_64-unknown-linux-gnu/aot/libwebrogue_aot_lib.a "$OUT_DIR"

clang main.c -nostdlib -c -o main.o


rm -f process_dump*
# strace -s 1000 -o process_dump -ff clang++ \
#     main.o \
#     ../aot_artifacts/x86_64-linux-gnu/libwebrogue_aot_lib.a \
#     empty.gnu.o \
#     -static-libstdc++ \
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
    /opt/rh/gcc-toolset-$GCC_VERSION/root/usr/lib/gcc/x86_64-redhat-linux/$GCC_VERSION/libstdc++.a \
    /opt/rh/gcc-toolset-$GCC_VERSION/root/usr/lib/gcc/x86_64-redhat-linux/$GCC_VERSION/libstdc++_nonshared.a

cp \
    /lib/../lib64/crt1.o \
    /lib/../lib64/crti.o \
    /opt/rh/gcc-toolset-$GCC_VERSION/root/usr/lib/gcc/x86_64-redhat-linux/$GCC_VERSION/crtbegin.o \
    /lib64/libm.so.6 \
    /lib/../lib64/libpthread.so \
    /lib/../lib64/libdl.so \
    /lib64/libgcc_s.so.1 \
    /opt/rh/gcc-toolset-$GCC_VERSION/root/usr/lib/gcc/x86_64-redhat-linux/$GCC_VERSION/libgcc.a \
    /lib64/libc.so.6 \
    /usr/lib64/libc_nonshared.a \
    /opt/rh/gcc-toolset-$GCC_VERSION/root/usr/lib/gcc/x86_64-redhat-linux/$GCC_VERSION/crtend.o \
    /lib/../lib64/crtn.o \
    "$OUT_DIR"

strip --strip-debug $OUT_DIR/libwebrogue_aot_lib.a

rm main.o

# ld.lld \
#     -pie \
#     --no-dependent-libraries \
#     --hash-style=gnu \
#     --build-id \
#     --eh-frame-hdr \
#     -m \
#     elf_x86_64 \
#     --strip-all \
#     --gc-sections \
#     -dynamic-linker \
#     /lib64/ld-linux-x86-64.so.2 \
#     -z \
#     relro \
#     -o \
#     aot \
#     --no-as-needed \
#     "$OUT_DIR/Scrt1.o" \
#     --no-as-needed \
#     "$OUT_DIR/crti.o" \
#     --no-as-needed \
#     "$OUT_DIR/crtbeginS.o" \
#     "$OUT_DIR/libwebrogue_aot_lib.a" \
#     empty.gnu.o \
#     "$OUT_DIR/libm.so.6" \
#     --as-needed \
#     "$OUT_DIR/libc.so.6" \
#     "$OUT_DIR/libgcc_s.so.1" \
#     "$OUT_DIR/libdl.so.2" \
#     "$OUT_DIR/libpthread.so.0" \
#     --no-as-needed \
#     "$OUT_DIR/crtendS.o" \
#     --no-as-needed \
#     "$OUT_DIR/crtn.o"

ld.lld \
    --hash-style=gnu \
    --build-id \
    --eh-frame-hdr \
    -m elf_x86_64 \
    -dynamic-linker /lib64/ld-linux-x86-64.so.2 \
    -o aot \
    "$OUT_DIR/crt1.o" \
    "$OUT_DIR/crti.o" \
    "$OUT_DIR/crtbegin.o" \
    "$OUT_DIR/libwebrogue_aot_lib.a" \
    "empty.gnu.o" \
    "$OUT_DIR/libm.so.6" \
    "$OUT_DIR/libpthread.so" \
    "$OUT_DIR/libdl.so" \
    "$OUT_DIR/libgcc_s.so.1" \
    "$OUT_DIR/libgcc.a" \
    "$OUT_DIR/libc.so.6" \
    "$OUT_DIR/libc_nonshared.a" \
    "$OUT_DIR/crtend.o" \
    "$OUT_DIR/crtn.o"

rm aot
