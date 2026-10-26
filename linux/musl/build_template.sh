cd $(dirname $(dirname $0))
set -ex

OUT_DIR="../aot_artifacts/$ARCH-linux-musl"
rm -rf "$OUT_DIR"

export NUM_JOBS=$(($(nproc) + 2))

export RUSTUP_TOOLCHAIN=nightly

TARGET_DIR=./musl/$ARCH/target
CARGO_FLAGS="--target-dir=$TARGET_DIR --target=$ARCH-unknown-linux-musl --profile aot -Z build-std"
mkdir -p "$OUT_DIR"
# rustup target add $ARCH-unknown-linux-musl

cargo build --manifest-path=../crates/aot-lib/Cargo.toml $CARGO_FLAGS
cp $TARGET_DIR/$ARCH-unknown-linux-musl/aot/libwebrogue_aot_lib.a "$OUT_DIR"

for VIRGL_LIB_TYPE in stub impl
do

    RLIB_PATH=$(cargo build \
        --manifest-path=../crates/virgl-lib/Cargo.toml \
        --features=$VIRGL_LIB_TYPE $CARGO_FLAGS \
        --message-format=json \
        | grep '"name":"webrogue_virgl_lib"' \
        | jq -r 'select(.reason == "compiler-artifact") | .filenames[] | select(endswith(".rlib"))')
    cp $RLIB_PATH "$OUT_DIR/libwebrogue_virgl_lib_$VIRGL_LIB_TYPE.a"
done

clang main.c -nostdlib -c -o main.o


rm -f process_dump*
# strace -s 1000 -o process_dump -ff \
clang \
    main.o \
    ../aot_artifacts/$ARCH-linux-musl/libwebrogue_aot_lib.a \
    ../aot_artifacts/$ARCH-linux-musl/libwebrogue_virgl_lib_impl.a \
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
    /usr/lib/libgcc_s.so.1 \
    /usr/lib/libc.so \
    "$OUT_DIR"

strip --strip-debug $OUT_DIR/*

rm main.o

# $1 = archive, $2 = output, $3 = virgl type.
link_command() {
    echo "ld.lld \
        -z now \
        -z relro \
        --hash-style=gnu \
        --build-id \
        --eh-frame-hdr \
        -dynamic-linker $INTERPRETER_PATH \
        -o $2 \
        --no-as-needed \
        $LLD_ARCH_ARGS \
        $OUT_DIR/crt1.o \
        $OUT_DIR/crti.o \
        $OUT_DIR/crtbeginT.o \
        $1 \
        $OUT_DIR/libwebrogue_virgl_lib_$3.a \
        empty.musl.$ARCH.o \
        $OUT_DIR/libgcc_s.so.1 \
        $OUT_DIR/libc.so \
        $OUT_DIR/crtend.o \
        $OUT_DIR/crtn.o"
}

python3 ../scripts/tree_shake.py \
    "$OUT_DIR/libwebrogue_aot_lib.a" \
    --output "$OUT_DIR/libwebrogue_aot_lib.a" \
    --machine "$ARCH" \
    --probe-command "$(link_command '{archive}' '{output}' impl)" \
    --probe-command "$(link_command '{archive}' '{output}' stub)"

for VIRGL_LIB_TYPE in stub impl
do
    eval "$(link_command "$OUT_DIR/libwebrogue_aot_lib.a" aot "$VIRGL_LIB_TYPE")"
    rm aot
done
