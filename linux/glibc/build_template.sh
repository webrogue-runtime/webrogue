cd $(dirname $(dirname $0))
set -ex

OUT_DIR="../aot_artifacts/$ARCH-linux-gnu"
# rm -rf "$OUT_DIR"

export NUM_JOBS=$(nproc)

TARGET_DIR=./glibc/$ARCH/target
CARGO_FLAGS="--target-dir=$TARGET_DIR --target=$ARCH-unknown-linux-gnu --profile aot"
mkdir -p "$OUT_DIR"
# rustup target add $ARCH-unknown-linux-gnu


cargo build --manifest-path=../crates/aot-lib/Cargo.toml $CARGO_FLAGS
cp $TARGET_DIR/$ARCH-unknown-linux-gnu/aot/libwebrogue_aot_lib.a "$OUT_DIR"

for GFXSTREAM_LIB_TYPE in stub impl
do
    cargo build --manifest-path=../crates/gfxstream-lib/Cargo.toml --features=$GFXSTREAM_LIB_TYPE $CARGO_FLAGS
    cp $TARGET_DIR/$ARCH-unknown-linux-gnu/aot/libwebrogue_gfxstream_lib.rlib  "$OUT_DIR/libwebrogue_gfxstream_lib_$GFXSTREAM_LIB_TYPE.a"
done

clang --gcc-install-dir=/opt/rh/gcc-toolset-$GCC_VERSION/root/usr/lib/gcc/$ARCH-redhat-linux/$GCC_VERSION main.c -nostdlib -c -o main.o


rm -f process_dump*
# strace -s 1000 -o process_dump -ff \
clang++ \
    -v \
    --gcc-install-dir=/opt/rh/gcc-toolset-$GCC_VERSION/root/usr/lib/gcc/$ARCH-redhat-linux/$GCC_VERSION \
    main.o \
    $OUT_DIR/libwebrogue_aot_lib.a \
    $OUT_DIR/libwebrogue_gfxstream_lib_impl.a \
    empty.gnu.$ARCH.o \
    -static-libstdc++ \
    -lm \
    -lpthread \
    -ldl \
    -o a.out \
    -fuse-ld=lld \
    -Wl,--threads=1 \
    >glibc/$ARCH/clang.out.log \
    2>glibc/$ARCH/clang.err.log

    # -Wl,--reproduce=reproduce.tar \

case "$ARCH" in
    x86_64)
        INTERPRETER_PATH=/lib64/ld-linux-x86_64.so.2
        LLD_ARCH_ARGS="-m elf_x86_64"
        ;;
    aarch64)
        INTERPRETER_PATH=/lib/ld-linux-aarch64.so.1
        cp $INTERPRETER_PATH "$OUT_DIR"
        LLD_ARCH_ARGS="-EL -m aarch64linux $OUT_DIR/ld-linux-aarch64.so.1"
        ;;
    *) 
        echo "Unsupported ARCH: $ARCH" >&2
        exit 1
        ;;
esac

llvm-ar q \
    "$OUT_DIR/libwebrogue_aot_lib.a" \
    "main.o"
    

llvm-ar qLs \
    "$OUT_DIR/libwebrogue_aot_lib.a" \
    /opt/rh/gcc-toolset-$GCC_VERSION/root/usr/lib/gcc/$ARCH-redhat-linux/$GCC_VERSION/libstdc++.a \
    /opt/rh/gcc-toolset-$GCC_VERSION/root/usr/lib/gcc/$ARCH-redhat-linux/$GCC_VERSION/libstdc++_nonshared.a

cp \
    /lib/../lib64/crt1.o \
    /lib/../lib64/crti.o \
    /opt/rh/gcc-toolset-$GCC_VERSION/root/usr/lib/gcc/$ARCH-redhat-linux/$GCC_VERSION/crtbegin.o \
    /lib64/libm.so.6 \
    /lib/../lib64/libpthread.so \
    /lib/../lib64/libdl.so \
    /lib64/libgcc_s.so.1 \
    /opt/rh/gcc-toolset-$GCC_VERSION/root/usr/lib/gcc/$ARCH-redhat-linux/$GCC_VERSION/libgcc.a \
    /lib64/libc.so.6 \
    /usr/lib64/libc_nonshared.a \
    /opt/rh/gcc-toolset-$GCC_VERSION/root/usr/lib/gcc/$ARCH-redhat-linux/$GCC_VERSION/crtend.o \
    /lib/../lib64/crtn.o \
    "$OUT_DIR"

strip --strip-debug $OUT_DIR/libwebrogue_aot_lib.a

# ld.lld \
#     -pie \
#     --no-dependent-libraries \
#     --hash-style=gnu \
#     --build-id \
#     --eh-frame-hdr \
#     -m \
#     elf_$ARCH \
#     --strip-all \
#     --gc-sections \
#     -dynamic-linker \
#     /lib64/ld-linux-$ARCH.so.2 \
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

for GFXSTREAM_LIB_TYPE in stub impl
do
    ld.lld \
        --hash-style=gnu \
        --build-id \
        --eh-frame-hdr \
        -dynamic-linker $INTERPRETER_PATH \
        -o aot \
        $LLD_ARCH_ARGS \
        "$OUT_DIR/crt1.o" \
        "$OUT_DIR/crti.o" \
        "$OUT_DIR/crtbegin.o" \
        "$OUT_DIR/libwebrogue_aot_lib.a" \
        "$OUT_DIR/libwebrogue_gfxstream_lib_$GFXSTREAM_LIB_TYPE.a" \
        "empty.gnu.$ARCH.o" \
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
done

rm main.o
