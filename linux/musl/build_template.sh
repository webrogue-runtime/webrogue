cd $(dirname $(dirname $0))
set -ex

COMMON_OUT_DIR="../aot_artifacts/linux-${ARCH}cpu-musllibc"
rm -rf "$COMMON_OUT_DIR"
mkdir -p "$COMMON_OUT_DIR"

cp \
    /usr/lib/crt1.o \
    /usr/lib/crti.o \
    /usr/lib/gcc/$ARCH-alpine-linux-musl/*/crtbeginT.o \
    /usr/lib/gcc/$ARCH-alpine-linux-musl/*/crtend.o \
    /usr/lib/crtn.o \
    /usr/lib/libgcc_s.so.1 \
    /usr/lib/libc.so \
    "$COMMON_OUT_DIR"
strip --strip-debug $COMMON_OUT_DIR/*

export RUSTUP_TOOLCHAIN=nightly
TARGET_DIR=./musl/$ARCH/target

clang main.c -nostdlib -c -o main.o

for VULKAN_TYPE in 'novulkan' 'vulkan'
do
    OUT_DIR="$COMMON_OUT_DIR-$VULKAN_TYPE"
    rm -rf "$OUT_DIR"
    mkdir -p "$OUT_DIR"
    CARGO_FLAGS="--target-dir=$TARGET_DIR --target=$ARCH-unknown-linux-musl --profile aot -Z build-std"
    if [ "$VULKAN_TYPE" = "vulkan" ]; then
        CARGO_FLAGS="$CARGO_FLAGS --features=vulkan"
    fi
    cargo build --manifest-path=../crates/aot-lib/Cargo.toml $CARGO_FLAGS
    cp $TARGET_DIR/$ARCH-unknown-linux-musl/aot/libwebrogue_aot_lib.a "$OUT_DIR"

    rm -f process_dump*
    # strace -s 1000 -o process_dump -ff \
    clang \
        main.o \
        $OUT_DIR/libwebrogue_aot_lib.a \
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

    # $1 = archive, $2 = output
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
            $COMMON_OUT_DIR/crt1.o \
            $COMMON_OUT_DIR/crti.o \
            $COMMON_OUT_DIR/crtbeginT.o \
            $1 \
            empty.musl.$ARCH.o \
            $COMMON_OUT_DIR/libgcc_s.so.1 \
            $COMMON_OUT_DIR/libc.so \
            $COMMON_OUT_DIR/crtend.o \
            $COMMON_OUT_DIR/crtn.o"
    }

    python3 ../scripts/tree_shake.py \
        "$OUT_DIR/libwebrogue_aot_lib.a" \
        --output "$OUT_DIR/libwebrogue_aot_lib.a" \
        --machine "$ARCH" \
        --probe-command "$(link_command '{archive}' '{output}')"

    eval "$(link_command "$OUT_DIR/libwebrogue_aot_lib.a" aot)"
    rm aot
done

rm main.o
