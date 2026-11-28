cd $(dirname $(dirname $0))
set -ex

COMMON_OUT_DIR="../aot_artifacts/linux-${ARCH}cpu-gnulibc"
rm -rf "$COMMON_OUT_DIR"
mkdir -p "$COMMON_OUT_DIR"

copy_stripped_so() {
    cp "$1/$2" "$COMMON_OUT_DIR/$2.u"

    IN_FILE="$1/$2"
    OUT_FILE="$COMMON_OUT_DIR/$2"
    SECTIONS_TO_SHRINK=(".text" ".rodata" ".data" ".init" ".fini" ".plt" ".plt.sec")
    OBJCOPY_ARGS=()
    for sec in "${SECTIONS_TO_SHRINK[@]}"; do
        if objdump -h "$IN_FILE" 2>/dev/null | grep -E " $sec " | grep -q "CONTENTS"; then
            OBJCOPY_ARGS+=("--update-section" "$sec=/dev/null")
        fi
    done

    if [ ${#OBJCOPY_ARGS[@]} -gt 0 ]; then
        objcopy "${OBJCOPY_ARGS[@]}" "$IN_FILE" "$OUT_FILE"
    else
        cp "$IN_FILE" "$OUT_FILE"
    fi

    strip --strip-unneeded "$OUT_FILE"
    rm "$COMMON_OUT_DIR/$2.u"
}

cp \
    /lib/../lib64/crt1.o \
    /lib/../lib64/crti.o \
    /opt/rh/gcc-toolset-$GCC_VERSION/root/usr/lib/gcc/$ARCH-redhat-linux/$GCC_VERSION/crtbegin.o \
    /opt/rh/gcc-toolset-$GCC_VERSION/root/usr/lib/gcc/$ARCH-redhat-linux/$GCC_VERSION/libgcc.a \
    /usr/lib64/libc_nonshared.a \
    /opt/rh/gcc-toolset-$GCC_VERSION/root/usr/lib/gcc/$ARCH-redhat-linux/$GCC_VERSION/crtend.o \
    /lib/../lib64/crtn.o \
    "$COMMON_OUT_DIR"

copy_stripped_so /lib64 libm.so.6
copy_stripped_so /lib64 libpthread.so
copy_stripped_so /lib64 libdl.so
copy_stripped_so /lib64 libgcc_s.so.1
copy_stripped_so /lib64 libc.so.6

export RUSTUP_TOOLCHAIN=nightly
TARGET_DIR=./gnu/$ARCH/target

clang --gcc-install-dir=/opt/rh/gcc-toolset-$GCC_VERSION/root/usr/lib/gcc/$ARCH-redhat-linux/$GCC_VERSION main.c -nostdlib -c -o main.o

for VULKAN_TYPE in 'novulkan' 'vulkan'
do
    OUT_DIR="$COMMON_OUT_DIR-$VULKAN_TYPE"
    rm -rf "$OUT_DIR"
    mkdir -p "$OUT_DIR"

    CARGO_FLAGS="--target-dir=$TARGET_DIR --target=$ARCH-unknown-linux-gnu --profile aot -Z build-std"
    if [ "$VULKAN_TYPE" = "vulkan" ]; then
        CARGO_FLAGS="$CARGO_FLAGS --features=vulkan"
    fi
    cargo build --manifest-path=../crates/aot-lib/Cargo.toml $CARGO_FLAGS
    cp $TARGET_DIR/$ARCH-unknown-linux-gnu/aot/libwebrogue_aot_lib.a "$OUT_DIR"

    rm -f process_dump*
    # strace -s 1000 -o process_dump -ff \
    clang \
        -v \
        --gcc-install-dir=/opt/rh/gcc-toolset-$GCC_VERSION/root/usr/lib/gcc/$ARCH-redhat-linux/$GCC_VERSION \
        main.o \
        $OUT_DIR/libwebrogue_aot_lib.a \
        empty.gnu.$ARCH.o \
        -static-libstdc++ \
        -lm \
        -lpthread \
        -ldl \
        -o a.out \
        -fuse-ld=lld \
        -Wl,--threads=1 \
        # >gnu/$ARCH/clang.out.log \
        # 2>gnu/$ARCH/clang.err.log \
        # -Wl,--reproduce=reproduce.tar \
    rm a.out

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

    strip --strip-debug $OUT_DIR/libwebrogue_aot_lib.a
    objcopy --remove-section=.eh_frame --remove-section=.eh_frame_hdr $OUT_DIR/libwebrogue_aot_lib.a

    # $1 = archive, $2 = output
    link_command() {
        echo "ld.lld \
            --hash-style=gnu \
            --build-id \
            --eh-frame-hdr \
            -dynamic-linker $INTERPRETER_PATH \
            -o $2 \
            $LLD_ARCH_ARGS \
            $COMMON_OUT_DIR/crt1.o \
            $COMMON_OUT_DIR/crti.o \
            $COMMON_OUT_DIR/crtbegin.o \
            $1 \
            empty.gnu.$ARCH.o \
            $COMMON_OUT_DIR/libm.so.6 \
            $COMMON_OUT_DIR/libpthread.so \
            $COMMON_OUT_DIR/libdl.so \
            $COMMON_OUT_DIR/libgcc_s.so.1 \
            $COMMON_OUT_DIR/libgcc.a \
            $COMMON_OUT_DIR/libc.so.6 \
            $COMMON_OUT_DIR/libc_nonshared.a \
            $COMMON_OUT_DIR/crtend.o \
            $COMMON_OUT_DIR/crtn.o"
    }

    # Stage 2: use the final link command as an oracle to remove objects that the
    # linker does not need.
    python3 ../scripts/tree_shake.py \
        "$OUT_DIR/libwebrogue_aot_lib.a" \
        --output "$OUT_DIR/libwebrogue_aot_lib.a" \
        --machine "$ARCH" \
        --probe-command "$(link_command '{archive}' '{output}')"

    eval "$(link_command "$OUT_DIR/libwebrogue_aot_lib.a" aot)"
    rm aot
done

rm main.o
