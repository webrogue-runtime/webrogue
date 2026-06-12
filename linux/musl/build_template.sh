cd $(dirname $(dirname $0))
set -ex

OUT_DIR="../aot_artifacts/$ARCH-linux-musl"
rm -rf "$OUT_DIR"

export NUM_JOBS=$(nproc)

TARGET_DIR=./musl/$ARCH/target
CARGO_FLAGS="--target-dir=$TARGET_DIR --target=$ARCH-unknown-linux-musl --profile aot"
mkdir -p "$OUT_DIR"
# rustup target add $ARCH-unknown-linux-musl

cargo build --manifest-path=../crates/aot-lib/Cargo.toml $CARGO_FLAGS
cp $TARGET_DIR/$ARCH-unknown-linux-musl/aot/libwebrogue_aot_lib.a "$OUT_DIR"

for GFXSTREAM_LIB_TYPE in stub impl
do
    cargo build --manifest-path=../crates/gfxstream-lib/Cargo.toml --features=$GFXSTREAM_LIB_TYPE $CARGO_FLAGS
    cp $TARGET_DIR/$ARCH-unknown-linux-musl/aot/libwebrogue_gfxstream_lib.rlib  "$OUT_DIR/libwebrogue_gfxstream_lib_$GFXSTREAM_LIB_TYPE.a"
done

clang main.c -nostdlib -c -o main.o


rm -f process_dump*
# strace -s 1000 -o process_dump -ff \
clang++ \
    main.o \
    ../aot_artifacts/$ARCH-linux-musl/libwebrogue_aot_lib.a \
    ../aot_artifacts/$ARCH-linux-musl/libwebrogue_gfxstream_lib_impl.a \
    empty.musl.$ARCH.o \
    -o a.out \
    -fuse-ld=lld \
    -Wl,--threads=1
rm a.out

case "$ARCH" in
    x86_64)
        INTERPRETER_PATH=/lib/ld-musl-x86_64.so.1
        LLD_ARCH_ARGS="-m elf_x86_64"
        ;;
    aarch64)
        INTERPRETER_PATH=/lib/ld-musl-aarch64.so.1
        cp $INTERPRETER_PATH "$OUT_DIR"
        LLD_ARCH_ARGS="-m aarch64linux"
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
    /usr/lib/libssp_nonshared.a \
    /usr/lib/gcc/$ARCH-alpine-linux-musl/*/libgcc.a

cp \
    /usr/lib/crt1.o \
    /usr/lib/crti.o \
    /usr/lib/gcc/$ARCH-alpine-linux-musl/*/crtbeginT.o \
    /usr/lib/gcc/$ARCH-alpine-linux-musl/*/crtend.o \
    /usr/lib/crtn.o \
    /usr/lib/libstdc++.so \
    /usr/lib/libgcc_s.so.1 \
    /usr/lib/libc.so \
    "$OUT_DIR"

strip --strip-debug $OUT_DIR/*

rm main.o

for GFXSTREAM_LIB_TYPE in stub impl
do
    ld.lld \
        -z now \
        -z relro \
        --hash-style=gnu \
        --build-id \
        --eh-frame-hdr \
        -dynamic-linker $INTERPRETER_PATH \
        -o aot \
        --no-as-needed \
        $LLD_ARCH_ARGS \
        "$OUT_DIR/crt1.o" \
        "$OUT_DIR/crti.o" \
        "$OUT_DIR/crtbeginT.o" \
        "$OUT_DIR/libwebrogue_aot_lib.a" \
        "$OUT_DIR/libwebrogue_gfxstream_lib_$GFXSTREAM_LIB_TYPE.a" \
        empty.musl.$ARCH.o \
        "$OUT_DIR/libstdc++.so" \
        "$OUT_DIR/libgcc_s.so.1" \
        "$OUT_DIR/libc.so" \
        "$OUT_DIR/crtend.o" \
        "$OUT_DIR/crtn.o"

    rm aot
done
