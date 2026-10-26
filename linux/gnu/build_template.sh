cd $(dirname $(dirname $0))
set -ex

OUT_DIR="../aot_artifacts/$ARCH-linux-gnu"
rm -rf "$OUT_DIR"

export NUM_JOBS=$(($(nproc) + 2))

export RUSTUP_TOOLCHAIN=nightly

TARGET_DIR=./gnu/$ARCH/target
CARGO_FLAGS="--target-dir=$TARGET_DIR --target=$ARCH-unknown-linux-gnu --profile aot -Z build-std"
mkdir -p "$OUT_DIR"
# rustup target add $ARCH-unknown-linux-gnu

cargo build --manifest-path=../crates/aot-lib/Cargo.toml $CARGO_FLAGS
cp $TARGET_DIR/$ARCH-unknown-linux-gnu/aot/libwebrogue_aot_lib.a "$OUT_DIR"

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

clang --gcc-install-dir=/opt/rh/gcc-toolset-$GCC_VERSION/root/usr/lib/gcc/$ARCH-redhat-linux/$GCC_VERSION main.c -nostdlib -c -o main.o


rm -f process_dump*
# strace -s 1000 -o process_dump -ff \
clang \
    -v \
    --gcc-install-dir=/opt/rh/gcc-toolset-$GCC_VERSION/root/usr/lib/gcc/$ARCH-redhat-linux/$GCC_VERSION \
    main.o \
    $OUT_DIR/libwebrogue_aot_lib.a \
    $OUT_DIR/libwebrogue_virgl_lib_impl.a \
    empty.gnu.$ARCH.o \
    -static-libstdc++ \
    -lm \
    -lpthread \
    -ldl \
    -o a.out \
    -fuse-ld=lld \
    -Wl,--threads=1 \
    >gnu/$ARCH/clang.out.log \
    2>gnu/$ARCH/clang.err.log

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

copy_stripped_so() {
    cp "$1/$2" "$OUT_DIR/$2.u"

    IN_FILE="$1/$2"
    OUT_FILE="$OUT_DIR/$2"
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
    rm "$OUT_DIR/$2.u"
}

cp \
    /lib/../lib64/crt1.o \
    /lib/../lib64/crti.o \
    /opt/rh/gcc-toolset-$GCC_VERSION/root/usr/lib/gcc/$ARCH-redhat-linux/$GCC_VERSION/crtbegin.o \
    /opt/rh/gcc-toolset-$GCC_VERSION/root/usr/lib/gcc/$ARCH-redhat-linux/$GCC_VERSION/libgcc.a \
    /usr/lib64/libc_nonshared.a \
    /opt/rh/gcc-toolset-$GCC_VERSION/root/usr/lib/gcc/$ARCH-redhat-linux/$GCC_VERSION/crtend.o \
    /lib/../lib64/crtn.o \
    "$OUT_DIR"

copy_stripped_so /lib64 libm.so.6
copy_stripped_so /lib64 libpthread.so
copy_stripped_so /lib64 libdl.so
copy_stripped_so /lib64 libgcc_s.so.1
copy_stripped_so /lib64 libc.so.6

strip --strip-debug $OUT_DIR/libwebrogue_aot_lib.a

for LIB in libwebrogue_aot_lib.a libwebrogue_virgl_lib_stub.a libwebrogue_virgl_lib_impl.a
do
    objcopy --remove-section=.eh_frame --remove-section=.eh_frame_hdr "$OUT_DIR/$LIB"
done

# $1 = archive, $2 = output, $3 = virgl type.
link_command() {
    echo "ld.lld \
        --hash-style=gnu \
        --build-id \
        --eh-frame-hdr \
        -dynamic-linker $INTERPRETER_PATH \
        -o $2 \
        $LLD_ARCH_ARGS \
        $OUT_DIR/crt1.o \
        $OUT_DIR/crti.o \
        $OUT_DIR/crtbegin.o \
        $1 \
        $OUT_DIR/libwebrogue_virgl_lib_$3.a \
        empty.gnu.$ARCH.o \
        $OUT_DIR/libm.so.6 \
        $OUT_DIR/libpthread.so \
        $OUT_DIR/libdl.so \
        $OUT_DIR/libgcc_s.so.1 \
        $OUT_DIR/libgcc.a \
        $OUT_DIR/libc.so.6 \
        $OUT_DIR/libc_nonshared.a \
        $OUT_DIR/crtend.o \
        $OUT_DIR/crtn.o"
}

# Stage 2: use the final link command as an oracle to remove objects that the
# linker does not need.
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

rm main.o
